<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Copy, Check, Trash2, FileImage, FileText, GitBranch, Download, Maximize, Minimize, FileDown, Loader2 } from 'lucide-svelte';
  import { marked, type RendererObject } from 'marked';
  import mermaid from 'mermaid';
  import { onMount } from 'svelte';
  import html2canvas from 'html2canvas';
  import { jsPDF } from 'jspdf';
  import { browser } from '$app/environment';
  import { page } from '$app/stores';
  
  // 动态导入 Tauri 插件（仅在需要时加载）
  let dialogModule: typeof import('@tauri-apps/plugin-dialog') | null = null;
  let fsModule: typeof import('@tauri-apps/plugin-fs') | null = null;

  let activeView = $state<'svg' | 'markdown' | 'mermaid'>('svg');
  
  // Check URL parameter for type
  $effect(() => {
    const typeParam = $page.url.searchParams.get('type');
    if (typeParam === 'svg' || typeParam === 'markdown' || typeParam === 'mermaid') {
      activeView = typeParam;
    }
  });
  let svgContent = $state('');
  let markdownContent = $state('');
  let mermaidContent = $state('');
  let copied = $state<{ svg: boolean; markdown: boolean; preview: boolean; mermaid: boolean }>({ svg: false, markdown: false, preview: false, mermaid: false });
  let previewElement = $state<HTMLDivElement | null>(null);
  let svgFileInput = $state<HTMLInputElement | null>(null);
  let markdownFileInput = $state<HTMLInputElement | null>(null);
  let mermaidFileInput = $state<HTMLInputElement | null>(null);
  let mermaidContainer = $state<HTMLDivElement | null>(null);
  let svgPreviewContainer = $state<HTMLDivElement | null>(null);
  let markdownPreviewContainer = $state<HTMLDivElement | null>(null);
  let mermaidPreviewContainer = $state<HTMLDivElement | null>(null);
  let fullscreenUpdateTrigger = $state(0);
  let fullscreenContainer = $state<HTMLElement | null>(null);
  let isTauri = $state(false);
  let escapeHandler: ((e: KeyboardEvent) => void) | null = null;
  let showCopyDropdown = $state(false);
  let isExporting = $state(false); // 导出状态
  
  // 行号相关的引用
  let svgTextareaRef = $state<HTMLTextAreaElement | null>(null);
  let svgLineNumbersRef = $state<HTMLDivElement | null>(null);
  let markdownTextareaRef = $state<HTMLTextAreaElement | null>(null);
  let markdownLineNumbersRef = $state<HTMLDivElement | null>(null);
  let mermaidTextareaRef = $state<HTMLTextAreaElement | null>(null);
  let mermaidLineNumbersRef = $state<HTMLDivElement | null>(null);
  
  // 更新行号显示
  function updateLineNumbers(textarea: HTMLTextAreaElement | null, lineNumbersDiv: HTMLDivElement | null) {
    if (!textarea || !lineNumbersDiv) return;
    
    const lines = textarea.value.split('\n');
    const lineCount = lines.length || 1;
    
    // 获取 textarea 的行高
    const computedStyle = window.getComputedStyle(textarea);
    const lineHeight = computedStyle.lineHeight;
    
    lineNumbersDiv.innerHTML = Array.from({ length: lineCount }, (_, i) => i + 1)
      .map(num => `<div class="px-2 text-right text-gray-500 dark:text-gray-400 text-sm select-none" style="line-height: ${lineHeight};">${num}</div>`)
      .join('');
  }
  
  // 同步滚动
  function syncScroll(textarea: HTMLTextAreaElement, lineNumbersDiv: HTMLDivElement) {
    lineNumbersDiv.scrollTop = textarea.scrollTop;
  }
  
  // 缩放相关状态
  let zoomLevel = $state(1.0);
  let minZoom = 0.5;
  let maxZoom = 5.0;
  let zoomStep = 0.1;

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  async function copyToClipboard(text: string, type: 'svg' | 'markdown' | 'preview') {
    if (!text) return;
    
    try {
      await navigator.clipboard.writeText(text);
      copied = { ...copied, [type]: true };
      setTimeout(() => {
        copied = { ...copied, [type]: false };
      }, 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  // CSS style for markdown preview copy - split into parts to avoid PostCSS parsing
  function getMarkdownPreviewStyle() {
    const parts = [
      'body{font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,Helvetica,Arial,sans-serif;line-height:1.7;color:#1f2937;}',
      'h1{font-size:1.875rem;font-weight:600;margin:0.75rem 0;}',
      'h2{font-size:1.5rem;font-weight:600;margin:0.75rem 0;}',
      'h3{font-size:1.25rem;font-weight:600;margin:0.5rem 0;}',
      'p{margin:0.5rem 0;}',
      'ul,ol{margin:0.5rem 0 0.5rem 1.25rem;}',
      // Inline code
      'code{background:#f3f4f6;padding:0.1rem 0.3rem;border-radius:0.25rem;font-family:SFMono-Regular,Menlo,Monaco,Consolas,"Liberation Mono","Courier New",monospace;font-size:0.9em;}',
      // Code block container - preserve whitespace and line breaks
      'pre{background:#e5e7eb;color:#1f2937;padding:0.75rem;border-radius:0.5rem;overflow:auto;margin:0.75rem 0;line-height:1.5;white-space:pre;word-wrap:normal;overflow-wrap:normal;font-family:SFMono-Regular,Menlo,Monaco,Consolas,"Liberation Mono","Courier New",monospace;border:1px solid #d1d5db;}',
      // Code inside pre (code block) - remove inline code styles
      'pre code{background:transparent;padding:0;border-radius:0;color:inherit;font-size:inherit;white-space:pre;word-wrap:normal;overflow-wrap:normal;display:block;overflow-x:auto;font-family:inherit;}',
      'blockquote{border-left:4px solid #9ca3af;padding-left:0.75rem;color:#4b5563;margin:0.5rem 0;}',
      'img{max-width:100%;border-radius:0.25rem;}',
      'table{border-collapse:collapse;width:100%;}',
      'th,td{border:1px solid #e5e7eb;padding:0.5rem;text-align:left;}',
      'a{color:#2563eb;text-decoration:none;}',
      'a:hover{text-decoration:underline;}'
    ];
    return parts.join('');
  }

  async function copyPreviewContent() {
    if (!markdownContent.trim() || !previewElement) return;
    const bodyHtml = previewElement.innerHTML;
    const plain = previewElement.innerText || previewElement.textContent || '';
    const style = getMarkdownPreviewStyle();
    // Use string concatenation instead of template literal to avoid PostCSS parsing
    const html = '<!DOCTYPE html><html><head><meta charset="utf-8"><' + 'style>' + style + '</' + 'style></head><body>' + bodyHtml + '</body></html>';
    try {
      const supportsWrite = typeof navigator.clipboard?.write === 'function';
      if (supportsWrite) {
        const item = new ClipboardItem({
          'text/html': new Blob([html], { type: 'text/html' }),
          'text/plain': new Blob([plain], { type: 'text/plain' })
        });
        await navigator.clipboard.write([item]);
      } else {
        const temp = document.createElement('div');
        temp.style.position = 'fixed';
        temp.style.left = '-9999px';
        temp.style.top = '0';
        temp.contentEditable = 'true';
        // Use string concatenation to avoid PostCSS parsing
        const styleContent = getMarkdownPreviewStyle();
        temp.innerHTML = '<' + 'style>' + styleContent + '</' + 'style>' + bodyHtml;
        document.body.appendChild(temp);
        const range = document.createRange();
        range.selectNodeContents(temp);
        const sel = window.getSelection();
        sel?.removeAllRanges();
        sel?.addRange(range);
        document.execCommand('copy');
        document.body.removeChild(temp);
      }
      copied = { ...copied, preview: true };
      setTimeout(() => {
        copied = { ...copied, preview: false };
      }, 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  async function copyPreviewAsHtml() {
    if (!markdownContent.trim() || !previewElement) return;
    const html = previewElement.innerHTML;
    const plain = previewElement.innerText || previewElement.textContent || '';
    try {
      const supportsWrite = typeof navigator.clipboard?.write === 'function';
      if (supportsWrite) {
        const item = new ClipboardItem({
          'text/html': new Blob([html], { type: 'text/html' }),
          'text/plain': new Blob([plain], { type: 'text/plain' })
        });
        await navigator.clipboard.write([item]);
      } else {
        const temp = document.createElement('div');
        temp.style.position = 'fixed';
        temp.style.left = '-9999px';
        temp.style.top = '0';
        temp.contentEditable = 'true';
        temp.innerHTML = html;
        document.body.appendChild(temp);
        const range = document.createRange();
        range.selectNodeContents(temp);
        const sel = window.getSelection();
        sel?.removeAllRanges();
        sel?.addRange(range);
        document.execCommand('copy');
        document.body.removeChild(temp);
      }
      copied = { ...copied, preview: true };
      setTimeout(() => {
        copied = { ...copied, preview: false };
      }, 2000);
    } catch (error) {
      console.error('Failed to copy HTML:', error);
    }
  }

  // 复制为微信公众号格式（带内联样式）
  async function copyForWechat() {
    if (!markdownContent.trim() || !previewElement) return;
    
    // 克隆预览元素
    const clone = previewElement.cloneNode(true) as HTMLElement;
    
    // 为微信公众号优化样式（内联样式）
    const applyWechatStyles = (element: HTMLElement) => {
      // 段落样式
      element.querySelectorAll('p').forEach((p: HTMLElement) => {
        p.style.cssText = 'margin: 1em 0; line-height: 1.75; font-size: 16px; color: #333;';
      });
      
      // 标题样式
      element.querySelectorAll('h1').forEach((h) => {
        (h as HTMLElement).style.cssText = 'font-size: 28px; font-weight: bold; margin: 1.5em 0 0.5em; color: #000; border-bottom: 2px solid #000; padding-bottom: 0.3em;';
      });
      element.querySelectorAll('h2').forEach((h) => {
        (h as HTMLElement).style.cssText = 'font-size: 24px; font-weight: bold; margin: 1.3em 0 0.5em; color: #000; border-bottom: 1px solid #ddd; padding-bottom: 0.3em;';
      });
      element.querySelectorAll('h3').forEach((h) => {
        (h as HTMLElement).style.cssText = 'font-size: 20px; font-weight: bold; margin: 1.2em 0 0.5em; color: #333;';
      });
      element.querySelectorAll('h4, h5, h6').forEach((h) => {
        (h as HTMLElement).style.cssText = 'font-size: 18px; font-weight: bold; margin: 1em 0 0.5em; color: #333;';
      });
      
      // 代码块样式
      element.querySelectorAll('pre').forEach((pre: HTMLElement) => {
        pre.style.cssText = 'background: #f6f8fa; border-radius: 6px; padding: 16px; overflow-x: auto; margin: 1em 0; border: 1px solid #e1e4e8;';
      });
      element.querySelectorAll('code').forEach((code: HTMLElement) => {
        if (!code.parentElement || code.parentElement.tagName !== 'PRE') {
          code.style.cssText = 'background: #f6f8fa; padding: 2px 6px; border-radius: 3px; font-family: Consolas, Monaco, monospace; font-size: 14px; color: #e83e8c;';
        } else {
          code.style.cssText = 'font-family: Consolas, Monaco, monospace; font-size: 14px; color: #24292e;';
        }
      });
      
      // 引用样式
      element.querySelectorAll('blockquote').forEach((bq: HTMLElement) => {
        bq.style.cssText = 'border-left: 4px solid #dfe2e5; padding-left: 16px; margin: 1em 0; color: #6a737d; font-style: italic;';
      });
      
      // 列表样式
      element.querySelectorAll('ul, ol').forEach((list) => {
        (list as HTMLElement).style.cssText = 'margin: 1em 0; padding-left: 2em; line-height: 1.75;';
      });
      element.querySelectorAll('li').forEach((li: HTMLElement) => {
        li.style.cssText = 'margin: 0.5em 0;';
      });
      
      // 链接样式
      element.querySelectorAll('a').forEach((a: HTMLElement) => {
        a.style.cssText = 'color: #0366d6; text-decoration: none; border-bottom: 1px solid #0366d6;';
      });
      
      // 图片样式
      element.querySelectorAll('img').forEach((img: HTMLElement) => {
        img.style.cssText = 'max-width: 100%; height: auto; display: block; margin: 1em auto; border-radius: 4px;';
      });
      
      // 表格样式
      element.querySelectorAll('table').forEach((table) => {
        (table as HTMLElement).style.cssText = 'border-collapse: collapse; width: 100%; margin: 1em 0; font-size: 14px;';
      });
      element.querySelectorAll('th, td').forEach((cell) => {
        (cell as HTMLElement).style.cssText = 'border: 1px solid #dfe2e5; padding: 8px 12px; text-align: left;';
      });
      element.querySelectorAll('th').forEach((th) => {
        (th as HTMLElement).style.cssText += ' background: #f6f8fa; font-weight: bold;';
      });
      
      // 分隔线样式
      element.querySelectorAll('hr').forEach((hr: HTMLElement) => {
        hr.style.cssText = 'border: none; border-top: 2px solid #eaecef; margin: 2em 0;';
      });
      
      // 强调样式
      element.querySelectorAll('strong, b').forEach((el) => {
        (el as HTMLElement).style.cssText = 'font-weight: bold; color: #000;';
      });
      element.querySelectorAll('em, i').forEach((el) => {
        (el as HTMLElement).style.cssText = 'font-style: italic; color: #333;';
      });
    };
    
    applyWechatStyles(clone);
    const html = clone.innerHTML;
    const plain = clone.innerText || clone.textContent || '';
    
    try {
      const supportsWrite = typeof navigator.clipboard?.write === 'function';
      if (supportsWrite) {
        const item = new ClipboardItem({
          'text/html': new Blob([html], { type: 'text/html' }),
          'text/plain': new Blob([plain], { type: 'text/plain' })
        });
        await navigator.clipboard.write([item]);
      } else {
        const temp = document.createElement('div');
        temp.style.position = 'fixed';
        temp.style.left = '-9999px';
        temp.style.top = '0';
        temp.contentEditable = 'true';
        temp.innerHTML = html;
        document.body.appendChild(temp);
        const range = document.createRange();
        range.selectNodeContents(temp);
        const sel = window.getSelection();
        sel?.removeAllRanges();
        sel?.addRange(range);
        document.execCommand('copy');
        document.body.removeChild(temp);
      }
      copied = { ...copied, preview: true };
      setTimeout(() => {
        copied = { ...copied, preview: false };
      }, 2000);
    } catch (error) {
      console.error('Failed to copy for Wechat:', error);
    }
  }

  function clear() {
    if (activeView === 'svg') {
      svgContent = '';
      if (svgFileInput) {
        svgFileInput.value = '';
      }
    } else if (activeView === 'markdown') {
      markdownContent = '';
      if (markdownFileInput) {
        markdownFileInput.value = '';
      }
    } else if (activeView === 'mermaid') {
      mermaidContent = '';
      if (mermaidFileInput) {
        mermaidFileInput.value = '';
      }
      if (mermaidContainer) {
        mermaidContainer.innerHTML = '';
      }
    }
    copied = { svg: false, markdown: false, preview: false, mermaid: false };
  }

  async function handleSvgFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    // 检查文件大小（限制为 10MB）
    const maxSize = 10 * 1024 * 1024; // 10MB
    if (file.size > maxSize) {
      alert(`File size exceeds 10MB limit. Please select a smaller file.`);
      target.value = '';
      return;
    }

    // 检查文件类型
    if (!file.type.includes('svg') && !file.name.toLowerCase().endsWith('.svg')) {
      alert('Please select an SVG file.');
      target.value = '';
      return;
    }

    try {
      const text = await file.text();
      svgContent = text;
    } catch (error) {
      console.error('Failed to read file:', error);
      alert('Failed to read file. Please try again.');
      target.value = '';
    }
  }

  async function handleMarkdownFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    // 检查文件大小（限制为 10MB）
    const maxSize = 10 * 1024 * 1024; // 10MB
    if (file.size > maxSize) {
      alert(`File size exceeds 10MB limit. Please select a smaller file.`);
      target.value = '';
      return;
    }

    // 检查文件类型
    const validExtensions = ['.md', '.markdown', '.txt'];
    const fileName = file.name.toLowerCase();
    const isValidType = validExtensions.some(ext => fileName.endsWith(ext)) || 
                        file.type.includes('markdown') || 
                        file.type === 'text/plain';
    
    if (!isValidType) {
      alert('Please select a Markdown file (.md, .markdown, or .txt).');
      target.value = '';
      return;
    }

    try {
      const text = await file.text();
      markdownContent = text;
    } catch (error) {
      console.error('Failed to read file:', error);
      alert('Failed to read file. Please try again.');
      target.value = '';
    }
  }

  async function handleMermaidFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    // 检查文件大小（限制为 10MB）
    const maxSize = 10 * 1024 * 1024; // 10MB
    if (file.size > maxSize) {
      alert(`File size exceeds 10MB limit. Please select a smaller file.`);
      target.value = '';
      return;
    }

    // 检查文件类型
    const validExtensions = ['.mmd', '.mermaid', '.md', '.txt'];
    const fileName = file.name.toLowerCase();
    const isValidType = validExtensions.some(ext => fileName.endsWith(ext)) || 
                        file.type === 'text/plain';
    
    if (!isValidType) {
      alert('Please select a Mermaid file (.mmd, .mermaid, .md, or .txt).');
      target.value = '';
      return;
    }

    try {
      const text = await file.text();
      mermaidContent = text;
      await renderMermaid();
    } catch (error) {
      console.error('Failed to read file:', error);
      alert('Failed to read file. Please try again.');
      target.value = '';
    }
  }

  async function renderMermaid() {
    if (!mermaidContainer || !mermaidContent.trim()) {
      return;
    }

    try {
      // 清空容器
      mermaidContainer.innerHTML = '';
      
      // 生成唯一 ID
      const id = `mermaid-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
      
      // 创建临时元素用于渲染
      const tempDiv = document.createElement('div');
      tempDiv.className = 'mermaid';
      tempDiv.id = id;
      tempDiv.textContent = mermaidContent;
      mermaidContainer.appendChild(tempDiv);
      
      // 使用 mermaid.run() 渲染图表（mermaid 11.x API）
      await mermaid.run({
        nodes: [tempDiv]
      });
    } catch (error) {
      console.error('Mermaid rendering error:', error);
      if (mermaidContainer) {
        mermaidContainer.innerHTML = `<div class="text-center text-red-600 dark:text-red-400 p-4">
          <p class="text-sm">Error rendering Mermaid diagram</p>
          <p class="text-xs mt-2">${error instanceof Error ? error.message : 'Unknown error'}</p>
        </div>`;
      }
    }
  }

  // 监听 mermaidContent 变化并重新渲染
  $effect(() => {
    if (activeView === 'mermaid' && mermaidContent.trim()) {
      renderMermaid();
    }
  });

  // 导出预览结果为图片
  async function exportPreviewAsImage() {
    let elementToExport: HTMLElement | null = null;
    let filename = 'preview';

    if (activeView === 'svg') {
      // SVG 预览：查找预览容器中的 SVG 元素
      const previewCard = document.querySelector('[data-preview="svg"]');
      if (previewCard) {
        const svgContainer = previewCard.querySelector('.border') as HTMLElement;
        if (svgContainer && svgContainer.querySelector('svg')) {
          elementToExport = svgContainer;
          filename = 'svg-preview';
        }
      }
    } else if (activeView === 'mermaid') {
      // Mermaid 预览：导出 Mermaid 容器
      // Mermaid 渲染后，.mermaid 元素会被转换为 SVG，所以检查 SVG 或 .mermaid
      if (mermaidContainer && (mermaidContainer.querySelector('svg') || mermaidContainer.querySelector('.mermaid'))) {
        elementToExport = mermaidContainer;
        filename = 'mermaid-preview';
      }
    }

    if (!elementToExport) {
      alert('No preview content to export.');
      return;
    }

    isExporting = true;
    try {
      // 使用 html2canvas 将元素转换为 canvas
      const canvas = await html2canvas(elementToExport, {
        backgroundColor: '#ffffff', // 白色背景
        scale: 2, // 提高分辨率
        useCORS: true,
        logging: false,
        allowTaint: true
      });

      // 检查是否在 Tauri 环境中
      const isTauri = browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

      if (isTauri) {
        // Tauri 环境：使用 dialog 和 fs API
        try {
          // 确保模块已加载
          if (!dialogModule) {
            dialogModule = await import('@tauri-apps/plugin-dialog');
          }
          if (!fsModule) {
            fsModule = await import('@tauri-apps/plugin-fs');
          }

          if (!dialogModule || !fsModule) {
            throw new Error('Failed to load Tauri plugins');
          }

          // 打开保存对话框
          // Tauri 2.0 插件使用命名导出
          const { save } = dialogModule;
          if (!save || typeof save !== 'function') {
            throw new Error('save function not found in dialog module');
          }
          
          const filePath = await save({
            defaultPath: `Kairoa-${filename}-${Date.now()}.png`,
            filters: [{
              name: 'PNG Image',
              extensions: ['png']
            }]
          });

          if (!filePath) {
            // 用户取消了保存对话框
            return;
          }

          // 将 canvas 转换为 blob，然后转换为 Uint8Array
          // 使用 Promise 包装 toBlob 以确保异步操作完成，并添加超时处理
          await new Promise<void>((resolve, reject) => {
            // 设置超时（10秒）
            const timeout = setTimeout(() => {
              reject(new Error('Timeout: Failed to generate image blob within 10 seconds.'));
            }, 10000);

            try {
              canvas.toBlob(async (blob) => {
                clearTimeout(timeout);
                
                try {
                  if (!blob) {
                    reject(new Error('Failed to generate image blob: blob is null.'));
                    return;
                  }

                  console.log('Image blob generated, size:', blob.size);

                  const arrayBuffer = await blob.arrayBuffer();
                  const uint8Array = new Uint8Array(arrayBuffer);

                  console.log('Writing file to:', filePath);
                  
                  // 写入文件 - Tauri 2.0 插件使用 writeFile（不是 writeBinaryFile）
                  const { writeFile } = fsModule;
                  if (!writeFile || typeof writeFile !== 'function') {
                    throw new Error('writeFile function not found in fs module');
                  }
                  
                  await writeFile(filePath, uint8Array);
                  
                  console.log('File saved successfully');
                  resolve();
                } catch (error) {
                  console.error('Error in toBlob callback:', error);
                  reject(error);
                }
              }, 'image/png');

              // 如果 canvas.toBlob 立即失败（某些浏览器可能不支持）
              if (typeof canvas.toBlob !== 'function') {
                clearTimeout(timeout);
                reject(new Error('canvas.toBlob is not supported'));
              }
            } catch (error) {
              clearTimeout(timeout);
              reject(error);
            }
          });
        } catch (error) {
          console.error('Tauri export error:', error);
          const errorMessage = error instanceof Error ? error.message : String(error);
          console.error('Error details:', {
            error,
            dialogModule: !!dialogModule,
            fsModule: !!fsModule,
            canvas: !!canvas,
            canvasWidth: canvas?.width,
            canvasHeight: canvas?.height
          });
          alert(`Failed to export image: ${errorMessage}`);
        }
      } else {
        // 浏览器环境：使用下载链接
        canvas.toBlob((blob) => {
          if (!blob) {
            alert('Failed to generate image.');
            return;
          }

          const url = URL.createObjectURL(blob);
          const link = document.createElement('a');
          link.href = url;
          link.download = `Kairoa-${filename}-${Date.now()}.png`;
          document.body.appendChild(link);
          link.click();
          document.body.removeChild(link);
          URL.revokeObjectURL(url);
        }, 'image/png');
      }
    } catch (error) {
      console.error('Export error:', error);
      alert('Failed to export image. Please try again.');
    } finally {
      isExporting = false;
    }
  }

  // 导出 Markdown 预览为 PDF
  async function exportMarkdownAsPdf() {
    if (!previewElement || !markdownContent.trim()) {
      alert('No Markdown content to export.');
      return;
    }

    isExporting = true;
    try {
      // 创建一个临时的全宽容器来渲染内容，确保格式正确
      // A4 内容宽度约 190mm，在 96 DPI 下约为 720px，我们使用 800px 以确保有足够空间
      const contentWidthPx = 800;
      
      const tempContainer = document.createElement('div');
      tempContainer.style.position = 'absolute';
      tempContainer.style.left = '-9999px';
      tempContainer.style.top = '0';
      tempContainer.style.width = `${contentWidthPx}px`;
      tempContainer.style.backgroundColor = '#ffffff';
      tempContainer.style.padding = '24px';
      tempContainer.style.boxSizing = 'border-box';
      
      // 克隆预览元素及其所有子元素
      const clonedElement = previewElement.cloneNode(true) as HTMLElement;
      
      // 确保克隆的元素有正确的样式
      clonedElement.style.width = '100%';
      clonedElement.style.maxWidth = '100%';
      clonedElement.style.margin = '0';
      clonedElement.style.padding = '0';
      
      // 确保 markdown-content 内的样式正确
      const markdownContentEl = clonedElement.querySelector('.markdown-content');
      if (markdownContentEl) {
        (markdownContentEl as HTMLElement).style.width = '100%';
        (markdownContentEl as HTMLElement).style.maxWidth = '100%';
      }
      
      // 添加到临时容器
      tempContainer.appendChild(clonedElement);
      document.body.appendChild(tempContainer);
      
      // 等待内容渲染和样式应用
      await new Promise(resolve => setTimeout(resolve, 200));

      // 使用 html2canvas 将预览内容转换为 canvas
      // 提高 scale 以获得更高清晰度（PDF 需要高分辨率）
      const canvas = await html2canvas(tempContainer, {
        backgroundColor: '#ffffff',
        scale: 3, // 提高分辨率到 3 倍
        useCORS: true,
        logging: false,
        allowTaint: true,
        width: tempContainer.scrollWidth,
        height: tempContainer.scrollHeight,
        windowWidth: tempContainer.scrollWidth,
        windowHeight: tempContainer.scrollHeight
      });

      // 清理临时容器
      document.body.removeChild(tempContainer);

      // A4 尺寸（毫米）
      const a4WidthMm = 210;
      const a4HeightMm = 297;
      const marginMm = 10; // 边距
      const contentWidthMm = a4WidthMm - marginMm * 2;
      const contentHeightMm = a4HeightMm - marginMm * 2;

      // 创建 PDF（使用毫米单位）
      const pdf = new jsPDF({
        orientation: 'portrait',
        unit: 'mm',
        format: 'a4',
        compress: true // 启用压缩但保持质量
      });

      // 计算图片尺寸和缩放比例
      const imgWidth = canvas.width;
      const imgHeight = canvas.height;
      
      // 使用更高的 DPI 进行计算（300 DPI 用于高质量打印）
      // 1mm = 11.81px at 300 DPI, 但我们的 canvas 是 96 DPI * scale
      // 所以需要正确转换：canvas 的像素 / scale = 原始像素
      const scale = 3; // 与 html2canvas 的 scale 保持一致
      const originalPxPerMm = 96 / 25.4; // 96 DPI 下的像素/毫米
      const scaledPxPerMm = originalPxPerMm * scale; // scale 倍后的像素/毫米
      
      // 计算缩放比例，使图片宽度适应内容区域（留边距）
      const imgWidthInMm = imgWidth / scaledPxPerMm;
      const widthRatio = contentWidthMm / imgWidthInMm;
      const imgWidthInPdf = contentWidthMm;
      const imgHeightInPdf = (imgHeight / scaledPxPerMm) * widthRatio;

      // 分页处理
      const pageContentHeight = contentHeightMm;
      let sourceY = 0; // 源图片的 Y 位置（像素）
      let remainingHeight = imgHeightInPdf; // 剩余高度（毫米）

      while (remainingHeight > 0) {
        // 如果不是第一页，添加新页面
        if (sourceY > 0) {
          pdf.addPage();
        }

        // 计算这一页要显示的高度
        const pageHeight = Math.min(pageContentHeight, remainingHeight);
        
        // 计算源图片中要裁剪的区域（像素）
        const sourceHeightPx = (pageHeight / imgHeightInPdf) * imgHeight;
        
        // 创建临时 canvas 来裁剪图片
        const tempCanvas = document.createElement('canvas');
        tempCanvas.width = imgWidth;
        tempCanvas.height = Math.ceil(sourceHeightPx);
        const tempCtx = tempCanvas.getContext('2d');
        if (tempCtx) {
          tempCtx.fillStyle = '#ffffff';
          tempCtx.fillRect(0, 0, tempCanvas.width, tempCanvas.height);
          tempCtx.drawImage(
            canvas,
            0, sourceY, imgWidth, sourceHeightPx, // 源区域
            0, 0, imgWidth, sourceHeightPx // 目标区域
          );
        }

        // 将裁剪后的图片添加到 PDF（添加边距）
        // 使用 PNG 格式和高质量压缩，确保清晰度
        pdf.addImage(
          tempCanvas.toDataURL('image/png', 1.0), // 使用最高质量
          'PNG',
          marginMm,
          marginMm,
          imgWidthInPdf,
          pageHeight,
          undefined,
          'FAST' // 快速渲染模式
        );

        // 更新位置
        sourceY += sourceHeightPx;
        remainingHeight -= pageHeight;
      }

      // 检查是否在 Tauri 环境中
      const isTauri = browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

      if (isTauri) {
        // Tauri 环境：使用 dialog 和 fs API
        try {
          if (!dialogModule) {
            dialogModule = await import('@tauri-apps/plugin-dialog');
          }
          if (!fsModule) {
            fsModule = await import('@tauri-apps/plugin-fs');
          }

          if (!dialogModule || !fsModule) {
            throw new Error('Failed to load Tauri plugins');
          }

          const { save } = dialogModule;
          if (!save || typeof save !== 'function') {
            throw new Error('save function not found in dialog module');
          }

          const filePath = await save({
            defaultPath: `Kairoa-markdown-preview-${Date.now()}.pdf`,
            filters: [{
              name: 'PDF Document',
              extensions: ['pdf']
            }]
          });

          if (!filePath) {
            return;
          }

          // 获取 PDF 的二进制数据
          const pdfData = pdf.output('arraybuffer');
          const uint8Array = new Uint8Array(pdfData);

          const { writeFile } = fsModule;
          if (!writeFile || typeof writeFile !== 'function') {
            throw new Error('writeFile function not found in fs module');
          }

          await writeFile(filePath, uint8Array);
        } catch (error) {
          console.error('Tauri export error:', error);
          const errorMessage = error instanceof Error ? error.message : String(error);
          alert(`Failed to export PDF: ${errorMessage}`);
        }
      } else {
        // 浏览器环境：直接下载
        pdf.save(`Kairoa-markdown-preview-${Date.now()}.pdf`);
      }
    } catch (error) {
      console.error('Export PDF error:', error);
      alert('Failed to export PDF. Please try again.');
    } finally {
      isExporting = false;
    }
  }

  // 全局键盘事件监听器，用于处理预览框内的全选
  let markdownSelectAllHandler: ((e: KeyboardEvent) => void) | null = null;

  // 初始化 Mermaid 和预加载 Tauri 插件
  onMount(() => {
    mermaid.initialize({ 
      startOnLoad: false,
      theme: 'default',
      securityLevel: 'loose',
      fontFamily: 'inherit'
    });
    
    // 预加载 Tauri 插件（如果可用）
    if (browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      import('@tauri-apps/plugin-dialog').then(module => {
        dialogModule = module;
      }).catch(() => {});
      import('@tauri-apps/plugin-fs').then(module => {
        fsModule = module;
      }).catch(() => {});
    }
  });

  // 检查 SVG 内容是否有效
  function isValidSVG(content: string): boolean {
    if (!content.trim()) return false;
    try {
      // 简单的 SVG 验证：检查是否包含 SVG 标签
      const parser = new DOMParser();
      const doc = parser.parseFromString(content, 'image/svg+xml');
      const parseError = doc.querySelector('parsererror');
      return !parseError && doc.querySelector('svg') !== null;
    } catch {
      return false;
    }
  }

  // 配置 marked 选项并自定义渲染器
  const customRenderer: Partial<RendererObject> = {
    link(token: any) {
      const href = token.href || '';
      const title = token.title || '';
      const text = token.text || '';
      return `<a href="${href}"${title ? ` title="${title}"` : ''}>${text}</a>`;
    },
    image(token: any) {
      const href = token.href || '';
      const title = token.title || '';
      const text = token.text || '';
      return `<img src="${href}" alt="${text}" class="max-w-full rounded"${title ? ` title="${title}"` : ''}>`;
    },
  };

  marked.use({
    breaks: true, // 支持换行符（单个换行符转换为 <br>）
    gfm: true, // 启用 GitHub Flavored Markdown
    renderer: customRenderer,
  });

  // 渲染 Markdown
  function renderMarkdown(content: string): string {
    if (!content.trim()) return '';
    
    try {
      return marked.parse(content) as string;
    } catch (error) {
      console.error('Markdown parsing error:', error);
      return `<p class="text-red-600 dark:text-red-400">Error parsing Markdown: ${error}</p>`;
    }
  }

  // 检查是否在 Tauri 环境中
  $effect(() => {
    if (browser && typeof window !== 'undefined') {
      isTauri = '__TAURI_INTERNALS__' in window;
    }
  });
  
  // 更新 SVG 行号
  $effect(() => {
    svgContent; // 监听内容变化
    // 使用 setTimeout 确保 DOM 已更新
    setTimeout(() => {
      updateLineNumbers(svgTextareaRef, svgLineNumbersRef);
    }, 0);
  });
  
  // 更新 Markdown 行号
  $effect(() => {
    markdownContent; // 监听内容变化
    // 使用 setTimeout 确保 DOM 已更新
    setTimeout(() => {
      updateLineNumbers(markdownTextareaRef, markdownLineNumbersRef);
    }, 0);
  });
  
  // 更新 Mermaid 行号
  $effect(() => {
    mermaidContent; // 监听内容变化
    // 使用 setTimeout 确保 DOM 已更新
    setTimeout(() => {
      updateLineNumbers(mermaidTextareaRef, mermaidLineNumbersRef);
    }, 0);
  });

  // 缩放控制函数
  function handleZoom(delta: number, container: HTMLElement) {
    const newZoom = Math.max(minZoom, Math.min(maxZoom, zoomLevel + delta));
    zoomLevel = newZoom;
    applyZoom(container);
    updateZoomDisplay(container);
  }

  function applyZoom(container: HTMLElement) {
    const previewContent = container.querySelector('.flex-1, .border, [class*="overflow-auto"]') as HTMLElement;
    if (!previewContent) return;

    // 查找需要缩放的元素
    const svgWrapper = previewContent.querySelector('div.max-w-full') as HTMLElement;
    const mermaidSvg = previewContent.querySelector('svg.mermaid, .mermaid svg, .mermaid') as HTMLElement;
    const markdownContent = previewContent.querySelector('.markdown-content') as HTMLElement;

    if (svgWrapper) {
      svgWrapper.style.transform = `scale(${zoomLevel})`;
      svgWrapper.style.transformOrigin = 'top center';
    } else if (mermaidSvg) {
      // Mermaid 使用用户的缩放级别（默认 100%）
      mermaidSvg.style.transform = `scale(${zoomLevel})`;
      mermaidSvg.style.transformOrigin = 'top center';
    } else if (markdownContent) {
      // Markdown 使用字体大小缩放
      const baseFontSize = 1.25; // rem
      markdownContent.style.fontSize = `${baseFontSize * zoomLevel}rem`;
    }
  }

  function resetZoom(container: HTMLElement) {
    zoomLevel = 1.0;
    applyZoom(container);
  }

  // 处理鼠标滚轮缩放
  function handleWheelZoom(e: WheelEvent, container: HTMLElement) {
    // 只在按住 Ctrl 或 Cmd 键时缩放
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      const delta = e.deltaY > 0 ? -zoomStep : zoomStep;
      handleZoom(delta, container);
    }
  }

  // 检查元素是否处于全屏状态
  function isElementFullscreen(element: HTMLElement | null): boolean {
    if (!element) return false;
    
    // 在 Tauri 环境中，检查 CSS 全屏状态
    if (isTauri) {
      return fullscreenContainer === element;
    }
    
    // 在浏览器环境中，检查 Fullscreen API
    const fullscreenElement =
      document.fullscreenElement ||
      (document as any).webkitFullscreenElement ||
      (document as any).mozFullScreenElement ||
      (document as any).msFullscreenElement;
    return fullscreenElement === element;
  }

  // 全屏切换功能
  async function toggleFullscreen(container: HTMLElement | null) {
    if (!container) return;

    try {
      const isCurrentlyFullscreen = isElementFullscreen(container);

      if (isTauri) {
        // Tauri 环境：使用 CSS 全屏覆盖
        if (!isCurrentlyFullscreen) {
          // 进入全屏：将容器设置为全屏覆盖
          fullscreenContainer = container;
          // 保存原始样式
          const originalClasses = container.className;
          container.setAttribute('data-original-classes', originalClasses);
          container.setAttribute('data-original-style', container.getAttribute('style') || '');
          
          // 设置为全屏覆盖
          container.style.position = 'fixed';
          container.style.top = '0';
          container.style.left = '0';
          container.style.right = '0';
          container.style.bottom = '0';
          container.style.width = '100vw';
          container.style.height = '100vh';
          container.style.zIndex = '9999';
          container.style.margin = '0';
          container.style.padding = '1rem';
          container.style.display = 'flex';
          container.style.flexDirection = 'column';
          container.style.justifyContent = 'flex-start';
          container.style.alignItems = 'flex-start';
          // 使用 Tailwind 的背景色类
          container.classList.add('bg-white', 'dark:bg-gray-900');
          
          // 创建并添加退出全屏按钮
          createExitFullscreenButton(container);
          
          // 创建缩放控制按钮
          createZoomControls(container);
          
          // 隐藏标题栏（如果存在）
          const titleBar = container.querySelector('.flex.items-center.justify-between.mb-2, [class*="mb-2"]') as HTMLElement;
          if (titleBar) {
            titleBar.setAttribute('data-original-display', titleBar.style.display || '');
            titleBar.style.display = 'none';
          }
          
          // 找到预览内容区域并确保其居中
          const previewContent = container.querySelector('.flex-1, .border, [class*="overflow-auto"]') as HTMLElement;
          if (previewContent) {
            // 保存预览内容区域的原始样式
            previewContent.setAttribute('data-original-style', previewContent.getAttribute('style') || '');
            
            // 设置全屏样式 - 只修改容器，不修改 SVG 本身
            previewContent.style.flex = '1';
            previewContent.style.width = '100%';
            previewContent.style.maxWidth = '100%';
            previewContent.style.height = '100%';
            previewContent.style.margin = '0';
            previewContent.style.border = 'none';
            previewContent.style.borderRadius = '0';
            previewContent.style.padding = '1rem';
            previewContent.style.boxSizing = 'border-box';
            previewContent.style.overflow = 'auto';
            previewContent.style.display = 'flex';
            previewContent.style.flexDirection = 'column';
            previewContent.style.alignItems = 'center';
            previewContent.style.justifyContent = 'flex-start';
            
            // 确保滚动到顶部
            setTimeout(() => {
              previewContent.scrollTop = 0;
            }, 0);
            
            // Markdown 内容 - 适度增大字体
            const markdownContent = previewContent.querySelector('.markdown-content');
            if (markdownContent) {
              (markdownContent as HTMLElement).setAttribute('data-original-style', (markdownContent as HTMLElement).getAttribute('style') || '');
              (markdownContent as HTMLElement).style.fontSize = '1.25rem';
              (markdownContent as HTMLElement).style.lineHeight = '1.8';
              (markdownContent as HTMLElement).style.maxWidth = '100%';
              (markdownContent as HTMLElement).style.width = '100%';
            }
            
            // Mermaid 内容 - 保持原始大小（100%）
            const mermaidSvg = previewContent.querySelector('svg.mermaid, .mermaid svg, .mermaid');
            if (mermaidSvg) {
              (mermaidSvg as HTMLElement).setAttribute('data-original-style', (mermaidSvg as HTMLElement).getAttribute('style') || '');
              if (mermaidSvg.tagName === 'SVG') {
                // 保持原始大小，让用户通过缩放控制调整
                (mermaidSvg as HTMLElement).style.maxWidth = '100%';
                (mermaidSvg as HTMLElement).style.maxHeight = '100%';
              }
            }
          }
          
          // 设置默认缩放级别为 120%
          zoomLevel = 1.2;
          
          // 应用初始缩放
          setTimeout(() => {
            applyZoom(container);
            updateZoomDisplay(container);
          }, 50);
          
          // 添加滚轮缩放监听
          const wheelHandler = (e: WheelEvent) => handleWheelZoom(e, container);
          previewContent.addEventListener('wheel', wheelHandler, { passive: false });
          container.setAttribute('data-wheel-handler', 'attached');
          
          // 添加 ESC 键监听
          escapeHandler = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && fullscreenContainer === container) {
              toggleFullscreen(container);
            }
          };
          document.addEventListener('keydown', escapeHandler);
        } else {
          // 退出全屏：恢复原始样式
          const originalClasses = container.getAttribute('data-original-classes');
          const originalStyle = container.getAttribute('data-original-style');
          
          // 移除全屏时添加的背景色类
          container.classList.remove('bg-white', 'dark:bg-gray-900');
          
          if (originalClasses) {
            container.className = originalClasses;
          }
          if (originalStyle) {
            container.setAttribute('style', originalStyle);
          } else {
            container.removeAttribute('style');
          }
          
          // 恢复标题栏显示
          const titleBar = container.querySelector('.flex.items-center.justify-between.mb-2, [class*="mb-2"]') as HTMLElement;
          if (titleBar) {
            const originalDisplay = titleBar.getAttribute('data-original-display');
            if (originalDisplay) {
              titleBar.style.display = originalDisplay;
            } else {
              titleBar.style.display = '';
            }
            titleBar.removeAttribute('data-original-display');
          }
          
          // 恢复预览内容区域的原始样式
          const previewContent = container.querySelector('.flex-1, .border, [class*="overflow-auto"]') as HTMLElement;
          if (previewContent) {
            const originalPreviewStyle = previewContent.getAttribute('data-original-style');
            if (originalPreviewStyle) {
              previewContent.setAttribute('style', originalPreviewStyle);
            } else {
              previewContent.removeAttribute('style');
            }
            previewContent.removeAttribute('data-original-style');
            
            // SVG 元素和 wrapper 没有被修改，所以不需要恢复
            
            // 恢复 Markdown 内容的原始样式
            const markdownContent = previewContent.querySelector('.markdown-content');
            if (markdownContent) {
              const originalStyle = (markdownContent as HTMLElement).getAttribute('data-original-style');
              if (originalStyle) {
                (markdownContent as HTMLElement).setAttribute('style', originalStyle);
              } else {
                (markdownContent as HTMLElement).removeAttribute('style');
              }
              (markdownContent as HTMLElement).removeAttribute('data-original-style');
              
            }
            
            // 恢复 Mermaid 内容的原始样式
            const mermaidSvg = previewContent.querySelector('svg.mermaid, .mermaid svg, .mermaid');
            if (mermaidSvg && (mermaidSvg as HTMLElement).hasAttribute('data-original-style')) {
              const originalStyle = (mermaidSvg as HTMLElement).getAttribute('data-original-style');
              if (originalStyle) {
                (mermaidSvg as HTMLElement).setAttribute('style', originalStyle);
              } else {
                (mermaidSvg as HTMLElement).removeAttribute('style');
              }
              (mermaidSvg as HTMLElement).removeAttribute('data-original-style');
            }
          }
          
          // 移除滚轮缩放监听
          if (previewContent && container.hasAttribute('data-wheel-handler')) {
            // 移除所有 wheel 事件监听器（通过克隆节点）
            const newPreviewContent = previewContent.cloneNode(true) as HTMLElement;
            previewContent.parentNode?.replaceChild(newPreviewContent, previewContent);
            container.removeAttribute('data-wheel-handler');
          }
          
          // 重置缩放级别
          zoomLevel = 1.0;
          
          // 移除缩放控制按钮
          removeZoomControls(container);
          
          // 移除退出全屏按钮
          removeExitFullscreenButton(container);
          
          // 移除 ESC 键监听
          if (escapeHandler) {
            document.removeEventListener('keydown', escapeHandler);
            escapeHandler = null;
          }
          
          fullscreenContainer = null;
        }
        fullscreenUpdateTrigger++;
      } else {
        // 浏览器环境：使用 Fullscreen API
        // 注意：放大逻辑在全屏状态变化监听器中处理，这里只负责切换全屏状态
        if (!isCurrentlyFullscreen) {
          // 进入全屏
          if (container.requestFullscreen) {
            await container.requestFullscreen();
          } else if ((container as any).webkitRequestFullscreen) {
            // Safari 支持
            await (container as any).webkitRequestFullscreen();
          } else if ((container as any).mozRequestFullScreen) {
            // Firefox 支持
            await (container as any).mozRequestFullScreen();
          } else if ((container as any).msRequestFullscreen) {
            // IE/Edge 支持
            await (container as any).msRequestFullscreen();
          }
        } else {
          // 退出全屏
          if (document.exitFullscreen) {
            await document.exitFullscreen();
          } else if ((document as any).webkitExitFullscreen) {
            await (document as any).webkitExitFullscreen();
          } else if ((document as any).mozCancelFullScreen) {
            await (document as any).mozCancelFullScreen();
          } else if ((document as any).msExitFullscreen) {
            await (document as any).msExitFullscreen();
          }
        }
        fullscreenUpdateTrigger++;
      }
    } catch (error) {
      console.error('Fullscreen error:', error);
    }
  }

  // 创建退出全屏按钮（Tauri 和浏览器环境通用）
  function createExitFullscreenButton(container: HTMLElement) {
    // 检查是否已存在退出按钮
    const existingBtn = container.querySelector('.exit-fullscreen-btn') || document.querySelector('.exit-fullscreen-btn');
    if (existingBtn) {
      return;
    }
    
    const exitBtn = document.createElement('button');
    // 使用与全屏按钮完全相同的样式：btn-secondary text-xs p-1.5
    // 确保在浏览器和 Tauri 环境下样式完全一致
    exitBtn.className = 'exit-fullscreen-btn fixed top-4 right-4 z-[10000] btn-secondary text-xs p-1.5';
    exitBtn.setAttribute('title', t('previewer.exitFullscreen'));
    exitBtn.setAttribute('aria-label', t('previewer.exitFullscreen'));
    // 使用 Minimize 图标（与全屏按钮一致，使用 lucide Minimize 图标的 SVG 路径）
    // 图标尺寸与全屏按钮完全一致：w-3.5 h-3.5 (14px)
    exitBtn.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-3.5 h-3.5"><path d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3"/></svg>`;
    exitBtn.onclick = (e) => {
      e.stopPropagation();
      toggleFullscreen(container);
    };
    
    // 将按钮添加到合适的容器
    // Tauri 环境：添加到容器内（因为使用 CSS 全屏覆盖）
    // 浏览器环境：添加到全屏元素内（Fullscreen API 会限制 fixed 定位的作用域）
    if (isTauri) {
      container.appendChild(exitBtn);
    } else {
      // 浏览器环境：添加到全屏元素内，确保 fixed 定位正确工作
      const fullscreenEl = 
        document.fullscreenElement ||
        (document as any).webkitFullscreenElement ||
        (document as any).mozFullScreenElement ||
        (document as any).msFullscreenElement;
      if (fullscreenEl) {
        (fullscreenEl as HTMLElement).appendChild(exitBtn);
      } else {
        // 如果还没有进入全屏，先添加到容器，全屏后会重新处理
        container.appendChild(exitBtn);
      }
    }
  }

  // 移除退出全屏按钮
  function removeExitFullscreenButton(container: HTMLElement) {
    const exitBtn = container.querySelector('.exit-fullscreen-btn') || document.querySelector('.exit-fullscreen-btn');
    if (exitBtn) {
      exitBtn.remove();
    }
  }

  // 创建缩放控制按钮
  function createZoomControls(container: HTMLElement) {
    // 检查是否已存在缩放控制
    const existingControls = container.querySelector('.zoom-controls') || document.querySelector('.zoom-controls');
    if (existingControls) {
      return;
    }
    
    const controlsDiv = document.createElement('div');
    controlsDiv.className = 'zoom-controls fixed bottom-4 right-4 z-[10000] flex flex-col gap-2 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 p-2';
    
    // 放大按钮
    const zoomInBtn = document.createElement('button');
    zoomInBtn.className = 'btn-secondary text-xs p-2 hover:bg-gray-100 dark:hover:bg-gray-700';
    zoomInBtn.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/><line x1="11" y1="8" x2="11" y2="14"/><line x1="8" y1="11" x2="14" y2="11"/></svg>`;
    zoomInBtn.title = 'Zoom In (Ctrl + Scroll)';
    zoomInBtn.onclick = (e) => {
      e.stopPropagation();
      handleZoom(zoomStep, container);
    };
    
    // 缩放级别显示
    const zoomDisplay = document.createElement('div');
    zoomDisplay.className = 'zoom-display text-xs text-center text-gray-700 dark:text-gray-300 font-mono px-2 py-1';
    zoomDisplay.textContent = `${Math.round(zoomLevel * 100)}%`;
    
    // 缩小按钮
    const zoomOutBtn = document.createElement('button');
    zoomOutBtn.className = 'btn-secondary text-xs p-2 hover:bg-gray-100 dark:hover:bg-gray-700';
    zoomOutBtn.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/><line x1="8" y1="11" x2="14" y2="11"/></svg>`;
    zoomOutBtn.title = 'Zoom Out (Ctrl + Scroll)';
    zoomOutBtn.onclick = (e) => {
      e.stopPropagation();
      handleZoom(-zoomStep, container);
    };
    
    // 重置按钮
    const resetBtn = document.createElement('button');
    resetBtn.className = 'btn-secondary text-xs p-2 hover:bg-gray-100 dark:hover:bg-gray-700';
    resetBtn.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/></svg>`;
    resetBtn.title = 'Reset Zoom (100%)';
    resetBtn.onclick = (e) => {
      e.stopPropagation();
      resetZoom(container);
      updateZoomDisplay(container);
    };
    
    controlsDiv.appendChild(zoomInBtn);
    controlsDiv.appendChild(zoomDisplay);
    controlsDiv.appendChild(zoomOutBtn);
    controlsDiv.appendChild(resetBtn);
    
    // 添加到容器
    if (isTauri) {
      container.appendChild(controlsDiv);
    } else {
      const fullscreenEl = 
        document.fullscreenElement ||
        (document as any).webkitFullscreenElement ||
        (document as any).mozFullScreenElement ||
        (document as any).msFullscreenElement;
      if (fullscreenEl) {
        (fullscreenEl as HTMLElement).appendChild(controlsDiv);
      } else {
        container.appendChild(controlsDiv);
      }
    }
  }

  // 移除缩放控制按钮
  function removeZoomControls(container: HTMLElement) {
    const controls = container.querySelector('.zoom-controls') || document.querySelector('.zoom-controls');
    if (controls) {
      controls.remove();
    }
  }

  // 更新缩放显示
  function updateZoomDisplay(container: HTMLElement) {
    const display = container.querySelector('.zoom-display') || document.querySelector('.zoom-display');
    if (display) {
      display.textContent = `${Math.round(zoomLevel * 100)}%`;
    }
  }

  // 准备全屏内容（浏览器环境）
  function prepareFullscreenContent(container: HTMLElement) {
    // 创建并添加退出全屏按钮
    createExitFullscreenButton(container);
    
    // 创建缩放控制按钮
    createZoomControls(container);
    
    // 隐藏标题栏
    const titleBar = container.querySelector('.flex.items-center.justify-between.mb-2, [class*="mb-2"]') as HTMLElement;
    if (titleBar) {
      titleBar.setAttribute('data-original-display', titleBar.style.display || '');
      titleBar.style.display = 'none';
    }
    
    // 找到预览内容区域
    const previewContent = container.querySelector('.flex-1, .border, [class*="overflow-auto"]') as HTMLElement;
    if (previewContent) {
      // 保存预览内容区域的原始样式
      previewContent.setAttribute('data-original-style', previewContent.getAttribute('style') || '');
      
      // 设置全屏样式 - 只修改容器，不修改 SVG 本身
      previewContent.style.padding = '1rem';
      previewContent.style.width = '100%';
      previewContent.style.height = '100%';
      previewContent.style.boxSizing = 'border-box';
      previewContent.style.overflow = 'auto';
      previewContent.style.display = 'flex';
      previewContent.style.flexDirection = 'column';
      previewContent.style.alignItems = 'center';
      previewContent.style.justifyContent = 'flex-start';
      
      // 确保滚动到顶部
      setTimeout(() => {
        previewContent.scrollTop = 0;
      }, 0);
      
      // 设置默认缩放级别为 120%
      zoomLevel = 1.2;
      
      // 应用初始缩放
      setTimeout(() => {
        applyZoom(container);
        updateZoomDisplay(container);
      }, 50);
      
      // 添加滚轮缩放监听
      const wheelHandler = (e: WheelEvent) => handleWheelZoom(e, container);
      previewContent.addEventListener('wheel', wheelHandler, { passive: false });
      container.setAttribute('data-wheel-handler', 'attached');
      
      // Markdown 内容 - 增大字体和宽度
      const markdownContent = previewContent.querySelector('.markdown-content');
      if (markdownContent) {
        (markdownContent as HTMLElement).setAttribute('data-original-style', (markdownContent as HTMLElement).getAttribute('style') || '');
        (markdownContent as HTMLElement).style.fontSize = '1.25rem';
        (markdownContent as HTMLElement).style.lineHeight = '1.8';
        (markdownContent as HTMLElement).style.maxWidth = '100%';
        (markdownContent as HTMLElement).style.width = '100%';
      }
      
      // Mermaid 内容 - 保持原始大小（100%）
      const mermaidSvg = previewContent.querySelector('svg.mermaid, .mermaid svg, .mermaid');
      if (mermaidSvg) {
        (mermaidSvg as HTMLElement).setAttribute('data-original-style', (mermaidSvg as HTMLElement).getAttribute('style') || '');
        if (mermaidSvg.tagName === 'SVG') {
          // 保持原始大小，让用户通过缩放控制调整
          (mermaidSvg as HTMLElement).style.maxWidth = '100%';
          (mermaidSvg as HTMLElement).style.maxHeight = '100%';
        }
      }
    }
  }

  // 恢复全屏内容（浏览器环境）
  function restoreFullscreenContent(container: HTMLElement) {
    // 移除滚轮缩放监听和恢复预览内容区域
    let previewContent = container.querySelector('.flex-1, .border, [class*="overflow-auto"]') as HTMLElement;
    if (previewContent && container.hasAttribute('data-wheel-handler')) {
      // 移除所有 wheel 事件监听器（通过克隆节点）
      const newPreviewContent = previewContent.cloneNode(true) as HTMLElement;
      previewContent.parentNode?.replaceChild(newPreviewContent, previewContent);
      container.removeAttribute('data-wheel-handler');
      // 更新引用
      previewContent = newPreviewContent;
    }
    
    // 重置缩放级别
    zoomLevel = 1.0;
    
    // 移除缩放控制按钮
    removeZoomControls(container);
    
    // 移除退出全屏按钮
    removeExitFullscreenButton(container);
    
    // 恢复标题栏显示
    const titleBar = container.querySelector('.flex.items-center.justify-between.mb-2, [class*="mb-2"]') as HTMLElement;
    if (titleBar) {
      const originalDisplay = titleBar.getAttribute('data-original-display');
      if (originalDisplay) {
        titleBar.style.display = originalDisplay;
      } else {
        titleBar.style.display = '';
      }
      titleBar.removeAttribute('data-original-display');
    }
    
    // 恢复预览内容区域的原始样式
    if (previewContent) {
      const originalPreviewStyle = previewContent.getAttribute('data-original-style');
      if (originalPreviewStyle) {
        previewContent.setAttribute('style', originalPreviewStyle);
      } else {
        previewContent.removeAttribute('style');
      }
      previewContent.removeAttribute('data-original-style');
      
      // 恢复 SVG 包装 div 的原始样式
      const svgWrapper = previewContent.querySelector('div.max-w-full') as HTMLElement;
      if (svgWrapper && svgWrapper.hasAttribute('data-original-style')) {
        const originalStyle = svgWrapper.getAttribute('data-original-style');
        if (originalStyle) {
          svgWrapper.setAttribute('style', originalStyle);
        } else {
          svgWrapper.removeAttribute('style');
        }
        svgWrapper.removeAttribute('data-original-style');
      }
      
      // 恢复 SVG 元素的原始样式
      const svgElement = previewContent.querySelector('svg');
      if (svgElement && svgElement.hasAttribute('data-original-width')) {
        const originalWidth = svgElement.getAttribute('data-original-width');
        const originalHeight = svgElement.getAttribute('data-original-height');
        const originalStyle = svgElement.getAttribute('data-original-style');
        const originalViewBox = svgElement.getAttribute('data-original-viewbox');
        if (originalWidth) svgElement.setAttribute('width', originalWidth);
        else svgElement.removeAttribute('width');
        if (originalHeight) svgElement.setAttribute('height', originalHeight);
        else svgElement.removeAttribute('height');
        if (originalViewBox) {
          svgElement.setAttribute('viewBox', originalViewBox);
        }
        if (originalStyle) svgElement.setAttribute('style', originalStyle);
        else svgElement.removeAttribute('style');
        svgElement.removeAttribute('data-original-width');
        svgElement.removeAttribute('data-original-height');
        svgElement.removeAttribute('data-original-viewbox');
      }
      
      // 恢复 Markdown 内容的原始样式
      const markdownContent = previewContent.querySelector('.markdown-content');
      if (markdownContent) {
        const originalStyle = (markdownContent as HTMLElement).getAttribute('data-original-style');
        if (originalStyle) {
          (markdownContent as HTMLElement).setAttribute('style', originalStyle);
        } else {
          (markdownContent as HTMLElement).removeAttribute('style');
        }
        (markdownContent as HTMLElement).removeAttribute('data-original-style');
      }
      
      // 恢复 Mermaid 内容的原始样式
      const mermaidSvg = previewContent.querySelector('svg.mermaid, .mermaid svg, .mermaid');
      if (mermaidSvg && (mermaidSvg as HTMLElement).hasAttribute('data-original-style')) {
        const originalStyle = (mermaidSvg as HTMLElement).getAttribute('data-original-style');
        if (originalStyle) {
          (mermaidSvg as HTMLElement).setAttribute('style', originalStyle);
        } else {
          (mermaidSvg as HTMLElement).removeAttribute('style');
        }
        (mermaidSvg as HTMLElement).removeAttribute('data-original-style');
      }
    }
  }

  // 监听全屏状态变化以更新 UI（仅浏览器环境）
  $effect(() => {
    if (!browser || isTauri) return;

    const handleFullscreenChange = () => {
      fullscreenUpdateTrigger++;
      
      // 检查当前全屏元素
      const fullscreenEl =
        document.fullscreenElement ||
        (document as any).webkitFullscreenElement ||
        (document as any).mozFullScreenElement ||
        (document as any).msFullscreenElement;
      
      if (fullscreenEl) {
        // 进入全屏后，应用放大样式
        prepareFullscreenContent(fullscreenEl as HTMLElement);
      } else {
        // 退出全屏后，恢复样式
        if (svgPreviewContainer) restoreFullscreenContent(svgPreviewContainer);
        if (markdownPreviewContainer) restoreFullscreenContent(markdownPreviewContainer);
        if (mermaidPreviewContainer) restoreFullscreenContent(mermaidPreviewContainer);
      }
    };

    document.addEventListener('fullscreenchange', handleFullscreenChange);
    document.addEventListener('webkitfullscreenchange', handleFullscreenChange);
    document.addEventListener('mozfullscreenchange', handleFullscreenChange);
    document.addEventListener('MSFullscreenChange', handleFullscreenChange);

    return () => {
      document.removeEventListener('fullscreenchange', handleFullscreenChange);
      document.removeEventListener('webkitfullscreenchange', handleFullscreenChange);
      document.removeEventListener('mozfullscreenchange', handleFullscreenChange);
      document.removeEventListener('MSFullscreenChange', handleFullscreenChange);
    };
  });
</script>

<div class="w-full ml-0 mr-0 p-2 flex flex-col h-[calc(100vh-2rem)]">
  <!-- 标签页导航 -->
  <div class="card p-0 mb-4 flex-shrink-0">
    <div class="flex items-center border-b border-gray-200 dark:border-gray-700">
      <button
        onclick={() => activeView = 'svg'}
        class="flex items-center gap-2 px-4 py-3 border-b-2 transition-colors {activeView === 'svg'
          ? 'border-primary-600 dark:border-primary-400 text-primary-600 dark:text-primary-400 bg-gray-50 dark:bg-gray-800'
          : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800'}"
      >
        <FileImage class="w-4 h-4" />
        <span class="text-sm font-medium">{t('previewer.svg')}</span>
      </button>
      <button
        onclick={() => activeView = 'markdown'}
        class="flex items-center gap-2 px-4 py-3 border-b-2 transition-colors {activeView === 'markdown'
          ? 'border-primary-600 dark:border-primary-400 text-primary-600 dark:text-primary-400 bg-gray-50 dark:bg-gray-800'
          : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800'}"
      >
        <FileText class="w-4 h-4" />
        <span class="text-sm font-medium">{t('previewer.markdown')}</span>
      </button>
      <button
        onclick={() => activeView = 'mermaid'}
        class="flex items-center gap-2 px-4 py-3 border-b-2 transition-colors {activeView === 'mermaid'
          ? 'border-primary-600 dark:border-primary-400 text-primary-600 dark:text-primary-400 bg-gray-50 dark:bg-gray-800'
          : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800'}"
      >
        <GitBranch class="w-4 h-4" />
        <span class="text-sm font-medium">{t('previewer.mermaid')}</span>
      </button>
    </div>
  </div>

  <!-- 内容区域 -->
  <div class="flex-1 min-h-0 flex flex-col">
    {#if activeView === 'svg'}
      <!-- SVG Preview - 左右布局 -->
      <div class="flex-1 min-h-0 flex flex-col">
        <div class="flex items-center justify-between mb-2 flex-shrink-0">
          <div class="flex items-center gap-4">
            <input
              type="file"
              accept=".svg,image/svg+xml"
              onchange={handleSvgFileSelect}
              bind:this={svgFileInput}
              class="hidden"
              id="svg-file-input"
            />
            <label
              for="svg-file-input"
              class="btn-secondary cursor-pointer flex items-center text-sm"
            >
              <FileImage class="w-4 h-4 inline mr-1" />
              {t('previewer.selectFile')}
            </label>
            <button
              onclick={() => copyToClipboard(svgContent, 'svg')}
              class="btn-secondary text-sm transition-all duration-200 {copied.svg ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
            >
              {#if copied.svg}
                <Check class="w-4 h-4 inline mr-1" />
                {t('common.copied')}
              {:else}
                <Copy class="w-4 h-4 inline mr-1" />
                {t('common.copy')}
              {/if}
            </button>
          </div>
          <div class="flex items-center gap-2">
            <button
              onclick={exportPreviewAsImage}
              class="btn-secondary text-sm"
              disabled={!svgContent.trim() || !isValidSVG(svgContent) || isExporting}
            >
              {#if isExporting}
                <Loader2 class="w-4 h-4 inline mr-1 animate-spin" />
                {t('previewer.exporting')}
              {:else}
                <Download class="w-4 h-4 inline mr-1" />
                {t('previewer.exportImage')}
              {/if}
            </button>
            <button
              onclick={clear}
              class="btn-secondary text-sm"
            >
              <Trash2 class="w-4 h-4 inline mr-1" />
              {t('common.clear')}
            </button>
          </div>
        </div>
        
        <div class="flex-1 min-h-0 grid grid-cols-2 gap-2 items-start">
          <!-- SVG 输入 -->
          <div class="card flex flex-col h-full">
            <div class="flex items-center justify-between mb-2 flex-shrink-0" style="height: 26px; min-height: 26px;">
              <div class="text-sm font-medium text-gray-700 dark:text-gray-300">
                {t('previewer.svgInput')}
              </div>
              <div class="w-0"></div>
            </div>
            <div class="flex-1 min-h-0 flex overflow-hidden">
              <div
                bind:this={svgLineNumbersRef}
                class="flex-shrink-0 overflow-hidden text-right bg-gray-50 dark:bg-gray-900/50 border-r border-gray-200 dark:border-gray-700 py-2"
                style="min-width: 3rem; max-height: 100%;"
              ></div>
              <textarea
                bind:this={svgTextareaRef}
                bind:value={svgContent}
                placeholder={t('previewer.svgPlaceholder')}
                class="textarea font-mono text-sm h-full resize-none flex-1 border-0 rounded-none"
                onscroll={(e) => {
                  if (svgLineNumbersRef) {
                    syncScroll(e.currentTarget, svgLineNumbersRef);
                  }
                }}
                oninput={() => {
                  updateLineNumbers(svgTextareaRef, svgLineNumbersRef);
                }}
              ></textarea>
            </div>
          </div>

          <!-- SVG 预览 -->
          <div class="card flex flex-col h-full" data-preview="svg" bind:this={svgPreviewContainer}>
            <div class="flex items-center justify-between mb-2 flex-shrink-0" style="height: 26px; min-height: 26px;">
              <div class="text-sm font-medium text-gray-700 dark:text-gray-300">
                {t('previewer.preview')}
              </div>
              <div class="flex items-center gap-2">
                {#if svgContent.trim() && !isValidSVG(svgContent)}
                  <span class="text-xs text-red-600 dark:text-red-400">
                    {t('previewer.invalidSVG')}
                  </span>
                {/if}
                <div class="relative group">
                  <button
                    onclick={() => toggleFullscreen(svgPreviewContainer)}
                    class="p-0.5 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-900 dark:text-gray-100 rounded transition-colors"
                    type="button"
                  >
                    {#if isElementFullscreen(svgPreviewContainer)}
                      <Minimize class="w-3.5 h-3.5" />
                    {:else}
                      <Maximize class="w-3.5 h-3.5" />
                    {/if}
                  </button>
                  <div class="absolute bottom-full right-0 mb-2 px-2 py-1 text-xs text-white bg-gray-900 dark:bg-gray-700 rounded shadow-lg opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap z-50">
                    {isElementFullscreen(svgPreviewContainer) ? t('previewer.exitFullscreen') : t('previewer.fullscreen')}
                    <div class="absolute top-full right-2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-900 dark:border-t-gray-700"></div>
                  </div>
                </div>
                <!-- 使用 fullscreenUpdateTrigger 来触发响应式更新 -->
                <span class="hidden">{fullscreenUpdateTrigger}</span>
              </div>
            </div>
            <div class="flex-1 border border-gray-300 dark:border-gray-600 rounded-lg overflow-auto bg-gray-50 dark:bg-gray-900 flex items-center justify-center p-4 min-h-0">
              {#if svgContent.trim()}
                {#if isValidSVG(svgContent)}
                  <div class="max-w-full max-h-full">
                    {@html svgContent}
                  </div>
                {:else}
                  <div class="text-center text-gray-400 dark:text-gray-500">
                    <p class="text-sm">{t('previewer.invalidSVGMessage')}</p>
                  </div>
                {/if}
              {:else}
                <div class="text-center text-gray-400 dark:text-gray-500 text-sm">
                  {t('previewer.svgPreviewPlaceholder')}
                </div>
              {/if}
            </div>
          </div>
        </div>
      </div>
    {:else if activeView === 'markdown'}
      <!-- Markdown Preview - 左右布局 -->
      <div class="flex-1 min-h-0 flex flex-col">
        <div class="flex items-center justify-between mb-2 flex-shrink-0">
          <div class="flex items-center gap-4">
            <input
              type="file"
              accept=".md,.markdown,.txt,text/markdown,text/plain"
              onchange={handleMarkdownFileSelect}
              bind:this={markdownFileInput}
              class="hidden"
              id="markdown-file-input"
            />
            <label
              for="markdown-file-input"
              class="btn-secondary cursor-pointer flex items-center text-sm"
            >
              <FileText class="w-4 h-4 inline mr-1" />
              {t('previewer.selectFile')}
            </label>
            <button
              onclick={clear}
              class="btn-secondary text-sm"
            >
              <Trash2 class="w-4 h-4 inline mr-1" />
              {t('common.clear')}
            </button>
          </div>
          <div class="relative">
            <button
              onclick={() => showCopyDropdown = !showCopyDropdown}
              class="btn-secondary text-sm transition-all duration-200 {copied.preview ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
              disabled={!markdownContent.trim()}
            >
              {#if copied.preview}
                <Check class="w-4 h-4 inline mr-1" />
                {t('common.copied')}
              {:else}
                <Copy class="w-4 h-4 inline mr-1" />
                {t('common.copy')}
              {/if}
              <svg class="w-3 h-3 inline ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
              </svg>
            </button>
            
            {#if showCopyDropdown}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div 
                class="fixed inset-0 z-10" 
                onclick={() => showCopyDropdown = false}
              ></div>
              <div class="absolute right-0 mt-1 w-48 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-20">
                <button
                  onclick={() => { copyPreviewContent(); showCopyDropdown = false; }}
                  class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-t-lg transition-colors"
                >
                  <Copy class="w-4 h-4 inline mr-2" />
                  {t('previewer.copyWithStyle')}
                </button>
                <button
                  onclick={() => { copyForWechat(); showCopyDropdown = false; }}
                  class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-b-lg transition-colors"
                >
                  <Copy class="w-4 h-4 inline mr-2" />
                  {t('previewer.copyForWechat')}
                </button>
              </div>
            {/if}
          </div>
        </div>
        
        <div class="flex-1 min-h-0 grid grid-cols-2 gap-2">
          <!-- Markdown 输入 -->
          <div class="card flex flex-col h-full">
            <div class="flex items-center justify-between mb-2 flex-shrink-0 h-[26px]">
              <div class="text-sm font-medium text-gray-700 dark:text-gray-300">
                {t('previewer.markdownInput')}
              </div>
            </div>
            <div class="flex-1 min-h-0 flex overflow-hidden">
              <div
                bind:this={markdownLineNumbersRef}
                class="flex-shrink-0 overflow-hidden text-right bg-gray-50 dark:bg-gray-900/50 border-r border-gray-200 dark:border-gray-700 py-2"
                style="min-width: 3rem; max-height: 100%;"
              ></div>
              <textarea
                bind:this={markdownTextareaRef}
                bind:value={markdownContent}
                placeholder={t('previewer.markdownPlaceholder')}
                class="textarea font-mono text-sm h-full resize-none flex-1 border-0 rounded-none"
                onscroll={(e) => {
                  if (markdownLineNumbersRef) {
                    syncScroll(e.currentTarget, markdownLineNumbersRef);
                  }
                }}
                oninput={() => {
                  updateLineNumbers(markdownTextareaRef, markdownLineNumbersRef);
                }}
              ></textarea>
            </div>
          </div>

          <!-- Markdown 预览 -->
          <div class="card flex flex-col h-full" bind:this={markdownPreviewContainer}>
            <div class="flex items-center justify-between mb-2 flex-shrink-0" style="height: 26px; min-height: 26px;">
              <div class="text-sm font-medium text-gray-700 dark:text-gray-300">
                {t('previewer.preview')}
              </div>
              <div class="flex items-center gap-1">
                {#if markdownContent.trim()}
                  <div class="relative group">
                    <button
                      onclick={exportMarkdownAsPdf}
                      disabled={isExporting}
                      class="p-0.5 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-900 dark:text-gray-100 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                      type="button"
                    >
                      {#if isExporting}
                        <Loader2 class="w-3.5 h-3.5 animate-spin" />
                      {:else}
                        <FileDown class="w-3.5 h-3.5" />
                      {/if}
                    </button>
                    <div class="absolute bottom-full right-0 mb-2 px-2 py-1 text-xs text-white bg-gray-900 dark:bg-gray-700 rounded shadow-lg opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap z-50">
                      {isExporting ? t('previewer.exporting') : t('previewer.exportPdf')}
                      <div class="absolute top-full right-2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-900 dark:border-t-gray-700"></div>
                    </div>
                  </div>
                {/if}
                <div class="relative group">
                  <button
                    onclick={() => toggleFullscreen(markdownPreviewContainer)}
                    class="p-0.5 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-900 dark:text-gray-100 rounded transition-colors"
                    type="button"
                  >
                    {#if isElementFullscreen(markdownPreviewContainer)}
                      <Minimize class="w-3.5 h-3.5" />
                    {:else}
                      <Maximize class="w-3.5 h-3.5" />
                    {/if}
                  </button>
                  <div class="absolute bottom-full right-0 mb-2 px-2 py-1 text-xs text-white bg-gray-900 dark:bg-gray-700 rounded shadow-lg opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap z-50">
                    {isElementFullscreen(markdownPreviewContainer) ? t('previewer.exitFullscreen') : t('previewer.fullscreen')}
                    <div class="absolute top-full right-2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-900 dark:border-t-gray-700"></div>
                  </div>
                </div>
              </div>
              <!-- 使用 fullscreenUpdateTrigger 来触发响应式更新 -->
              <span class="hidden">{fullscreenUpdateTrigger}</span>
            </div>
            <div 
              class="flex-1 border border-gray-300 dark:border-gray-600 rounded-lg overflow-auto bg-white dark:bg-gray-800 p-6 min-h-0"
              tabindex="0"
              onclick={(e) => {
                // 当点击预览框时，让预览框获得焦点
                if (previewElement && e.target instanceof Node && previewElement.contains(e.target)) {
                  e.currentTarget.focus();
                }
              }}
              onkeydown={(e) => {
                // 只在预览框内处理 Ctrl+A / Cmd+A
                if ((e.ctrlKey || e.metaKey) && (e.key === 'a' || e.key === 'A')) {
                  if (previewElement && markdownContent.trim()) {
                    e.preventDefault();
                    e.stopPropagation();
                    
                    // 选中预览框内的所有内容
                    const range = document.createRange();
                    range.selectNodeContents(previewElement);
                    const selection = window.getSelection();
                    if (selection) {
                      selection.removeAllRanges();
                      selection.addRange(range);
                    }
                  }
                }
              }}
            >
              {#if markdownContent.trim()}
                <div class="markdown-content" bind:this={previewElement}>
                  {@html renderMarkdown(markdownContent)}
                </div>
              {:else}
                <div class="flex items-center justify-center h-full text-gray-400 dark:text-gray-500 text-sm">
                  {t('previewer.markdownPreviewPlaceholder')}
                </div>
              {/if}
            </div>
          </div>
        </div>
      </div>
    {:else if activeView === 'mermaid'}
      <!-- Mermaid Preview - 左右布局 -->
      <div class="flex-1 min-h-0 flex flex-col">
        <div class="flex items-center justify-between mb-2 flex-shrink-0">
          <div class="flex items-center gap-4">
            <input
              type="file"
              accept=".mmd,.mermaid,.md,.txt,text/plain"
              onchange={handleMermaidFileSelect}
              bind:this={mermaidFileInput}
              class="hidden"
              id="mermaid-file-input"
            />
            <label
              for="mermaid-file-input"
              class="btn-secondary cursor-pointer flex items-center text-sm"
            >
              <GitBranch class="w-4 h-4 inline mr-1" />
              {t('previewer.selectFile')}
            </label>
            <button
              onclick={() => copyToClipboard(mermaidContent, 'mermaid')}
              class="btn-secondary text-sm transition-all duration-200 {copied.mermaid ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
            >
              {#if copied.mermaid}
                <Check class="w-4 h-4 inline mr-1" />
                {t('common.copied')}
              {:else}
                <Copy class="w-4 h-4 inline mr-1" />
                {t('common.copy')}
              {/if}
            </button>
          </div>
          <div class="flex items-center gap-2">
            <button
              onclick={exportPreviewAsImage}
              class="btn-secondary text-sm"
              disabled={!mermaidContent.trim() || isExporting}
            >
              {#if isExporting}
                <Loader2 class="w-4 h-4 inline mr-1 animate-spin" />
                {t('previewer.exporting')}
              {:else}
                <Download class="w-4 h-4 inline mr-1" />
                {t('previewer.exportImage')}
              {/if}
            </button>
            <button
              onclick={clear}
              class="btn-secondary text-sm"
            >
              <Trash2 class="w-4 h-4 inline mr-1" />
              {t('common.clear')}
            </button>
          </div>
        </div>
        
        <div class="flex-1 min-h-0 grid grid-cols-2 gap-2 items-start">
          <!-- Mermaid 输入 -->
          <div class="card flex flex-col h-full">
            <div class="flex items-center justify-between mb-2 flex-shrink-0" style="height: 26px; min-height: 26px;">
              <div class="text-sm font-medium text-gray-700 dark:text-gray-300">
                {t('previewer.mermaidInput')}
              </div>
              <div class="w-0"></div>
            </div>
            <div class="flex-1 min-h-0 flex overflow-hidden">
              <div
                bind:this={mermaidLineNumbersRef}
                class="flex-shrink-0 overflow-hidden text-right bg-gray-50 dark:bg-gray-900/50 border-r border-gray-200 dark:border-gray-700 py-2"
                style="min-width: 3rem; max-height: 100%;"
              ></div>
              <textarea
                bind:this={mermaidTextareaRef}
                bind:value={mermaidContent}
                placeholder={t('previewer.mermaidPlaceholder')}
                class="textarea font-mono text-sm h-full resize-none flex-1 border-0 rounded-none"
                onscroll={(e) => {
                  if (mermaidLineNumbersRef) {
                    syncScroll(e.currentTarget, mermaidLineNumbersRef);
                  }
                }}
                oninput={() => {
                  updateLineNumbers(mermaidTextareaRef, mermaidLineNumbersRef);
                }}
              ></textarea>
            </div>
          </div>

          <!-- Mermaid 预览 -->
          <div class="card flex flex-col h-full" bind:this={mermaidPreviewContainer}>
            <div class="flex items-center justify-between mb-2 flex-shrink-0" style="height: 26px; min-height: 26px;">
              <div class="text-sm font-medium text-gray-700 dark:text-gray-300">
                {t('previewer.preview')}
              </div>
              <div class="relative group">
                <button
                  onclick={() => toggleFullscreen(mermaidPreviewContainer)}
                  class="p-0.5 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-900 dark:text-gray-100 rounded transition-colors"
                  type="button"
                >
                  {#if isElementFullscreen(mermaidPreviewContainer)}
                    <Minimize class="w-3.5 h-3.5" />
                  {:else}
                    <Maximize class="w-3.5 h-3.5" />
                  {/if}
                </button>
                <div class="absolute bottom-full right-0 mb-2 px-2 py-1 text-xs text-white bg-gray-900 dark:bg-gray-700 rounded shadow-lg opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap z-50">
                  {isElementFullscreen(mermaidPreviewContainer) ? t('previewer.exitFullscreen') : t('previewer.fullscreen')}
                  <div class="absolute top-full right-2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-900 dark:border-t-gray-700"></div>
                </div>
              </div>
              <!-- 使用 fullscreenUpdateTrigger 来触发响应式更新 -->
              <span class="hidden">{fullscreenUpdateTrigger}</span>
            </div>
            <div class="flex-1 border border-gray-300 dark:border-gray-600 rounded-lg overflow-auto bg-white dark:bg-gray-800 p-6 min-h-0" bind:this={mermaidContainer}>
              {#if !mermaidContent.trim()}
                <div class="flex items-center justify-center h-full text-gray-400 dark:text-gray-500 text-sm">
                  {t('previewer.mermaidPreviewPlaceholder')}
                </div>
              {/if}
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

