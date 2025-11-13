<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { RotateCw, Download, Upload } from 'lucide-svelte';
  import { browser } from '$app/environment';
  import { page } from '$app/stores';
  
  type ToolType = 'rotate' | 'scale' | 'convert';
  
  let toolType = $state<ToolType>('rotate');
  
  // Check URL parameter for type
  $effect(() => {
    const typeParam = $page.url.searchParams.get('type');
    if (typeParam === 'rotate' || typeParam === 'scale' || typeParam === 'convert') {
      toolType = typeParam as ToolType;
    }
  });
  let imageFile = $state<File | null>(null);
  let imageUrl = $state<string>('');
  let rotationAngle = $state<number>(0); // 用户输入的角度
  let realTimeRotation = $state<number>(0); // 实时预览角度
  let processedImageUrl = $state<string>('');
  let isProcessing = $state(false);
  let error = $state('');
  let successMessage = $state('');
  let isDragging = $state(false); // 拖拽状态
  
  // Scale related state
  let originalWidth = $state<number>(0);
  let originalHeight = $state<number>(0);
  let scaleWidth = $state<number>(0);
  let scaleHeight = $state<number>(0);
  let scalePercentage = $state<number>(100); // 缩放百分比
  let maintainAspectRatio = $state<boolean>(true); // 保持宽高比
  
  // Convert related state
  type ImageFormat = 'png' | 'jpg' | 'webp' | 'gif';
  let targetFormat = $state<ImageFormat>('png');
  let convertedImageUrl = $state<string>('');
  let convertedImageBlob = $state<Blob | null>(null);
  
  // Tauri API
  let saveFn: ((options: any) => Promise<string | null>) | null = $state(null);
  let writeFileFn: ((path: string, contents: Uint8Array) => Promise<void>) | null = $state(null);
  let isTauriAvailable = $state(false);
  
  if (browser) {
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      isTauriAvailable = true;
      Promise.all([
        import('@tauri-apps/plugin-dialog'),
        import('@tauri-apps/plugin-fs')
      ]).then(([dialogModule, fsModule]) => {
        saveFn = dialogModule.save;
        writeFileFn = fsModule.writeFile;
      }).catch((err) => {
        console.error('Failed to load Tauri APIs:', err);
        isTauriAvailable = false;
      });
    }
  }
  
  let translations = $derived($translationsStore);
  
  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }
  
  function switchToolType(type: ToolType) {
    toolType = type;
    clearImage();
  }
  
  function handleFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    
    if (file) {
      if (!file.type.startsWith('image/')) {
        error = t('imageTools.invalidImageType');
        // 重置 input value，以便可以重新选择
        target.value = '';
        return;
      }
      
      imageFile = file;
      error = '';
      
      // Create preview URL
      if (imageUrl) {
        URL.revokeObjectURL(imageUrl);
      }
      imageUrl = URL.createObjectURL(file);
      
      // Clear processed image
      if (processedImageUrl) {
        URL.revokeObjectURL(processedImageUrl);
        processedImageUrl = '';
      }
      
      // Reset rotation
      rotationAngle = 0;
      realTimeRotation = 0;
      
      // Load image dimensions for scale
      const img = new Image();
      img.onload = () => {
        originalWidth = img.width;
        originalHeight = img.height;
        scaleWidth = img.width;
        scaleHeight = img.height;
        scalePercentage = 100;
      };
      img.src = imageUrl;
      
      // 重置 input value，以便可以再次选择同一张图片
      target.value = '';
    }
  }
  
  function handleDragEnter(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    
    // 检查是否拖拽的是文件
    // 在 Tauri 中，可能需要检查 files 数组而不是 types
    const hasFiles = event.dataTransfer?.files?.length > 0;
    
    // 如果没有 files，检查 types（浏览器环境）
    if (!hasFiles) {
      const types = event.dataTransfer?.types || [];
      const hasFileTypes = types.includes('Files') || 
                          types.includes('application/x-moz-file') ||
                          types.some(type => type.includes('file'));
      if (hasFileTypes) {
        isDragging = true;
        if (event.dataTransfer) {
          event.dataTransfer.dropEffect = 'copy';
        }
      }
    } else {
      // 有文件，直接设置拖拽状态
      isDragging = true;
      if (event.dataTransfer) {
        event.dataTransfer.dropEffect = 'copy';
      }
    }
  }
  
  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    // 只有当离开整个容器时才取消拖拽状态
    const target = event.currentTarget as HTMLElement;
    const relatedTarget = event.relatedTarget as HTMLElement;
    if (!target.contains(relatedTarget)) {
      isDragging = false;
    }
  }
  
  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    // 设置拖拽效果为 copy
    if (event.dataTransfer) {
      // 在 Tauri 中，需要检查是否有文件
      const hasFiles = event.dataTransfer.files?.length > 0;
      if (hasFiles || event.dataTransfer.types?.length > 0) {
        event.dataTransfer.dropEffect = 'copy';
      }
    }
  }
  
  function handleDrop(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    isDragging = false;
    
    // 调试信息
    console.log('Drop event:', {
      files: event.dataTransfer?.files?.length,
      types: event.dataTransfer?.types,
      items: event.dataTransfer?.items?.length
    });
    
    const file = event.dataTransfer?.files[0];
    
    if (file) {
      // 在 Tauri 中，文件类型可能为空，需要检查文件扩展名
      const fileName = file.name.toLowerCase();
      const imageExtensions = ['.jpg', '.jpeg', '.png', '.gif', '.webp', '.bmp', '.svg', '.ico', '.tiff', '.tif'];
      const isImageFile = file.type.startsWith('image/') || 
                         imageExtensions.some(ext => fileName.endsWith(ext));
      
      if (!isImageFile) {
        error = t('imageTools.invalidImageType');
        return;
      }
      
      imageFile = file;
      error = '';
      
      if (imageUrl) {
        URL.revokeObjectURL(imageUrl);
      }
      imageUrl = URL.createObjectURL(file);
      
      if (processedImageUrl) {
        URL.revokeObjectURL(processedImageUrl);
        processedImageUrl = '';
      }
      
      // Reset rotation
      rotationAngle = 0;
      realTimeRotation = 0;
      
      // Load image dimensions for scale
      const img = new Image();
      img.onload = () => {
        originalWidth = img.width;
        originalHeight = img.height;
        scaleWidth = img.width;
        scaleHeight = img.height;
        scalePercentage = 100;
      };
      img.src = imageUrl;
    } else {
      // 如果没有文件，可能是拖拽了其他内容
      console.warn('No file in drop event', {
        dataTransfer: event.dataTransfer,
        types: event.dataTransfer?.types,
        items: event.dataTransfer?.items
      });
    }
  }
  
  // 当旋转角度变化时，更新实时预览
  $effect(() => {
    if (imageUrl) {
      realTimeRotation = rotationAngle;
    }
  });
  
  async function rotateImage() {
    if (!imageFile) {
      error = t('imageTools.noImageSelected');
      return;
    }
    
    isProcessing = true;
    error = '';
    
    try {
      const img = new Image();
      img.src = imageUrl;
      
      await new Promise((resolve, reject) => {
        img.onload = resolve;
        img.onerror = reject;
      });
      
      const canvas = document.createElement('canvas');
      const ctx = canvas.getContext('2d');
      
      if (!ctx) {
        throw new Error('Failed to get canvas context');
      }
      
      // Calculate new dimensions based on rotation
      const angle = (rotationAngle * Math.PI) / 180;
      const cos = Math.abs(Math.cos(angle));
      const sin = Math.abs(Math.sin(angle));
      
      canvas.width = img.width * cos + img.height * sin;
      canvas.height = img.width * sin + img.height * cos;
      
      // Rotate and draw
      ctx.translate(canvas.width / 2, canvas.height / 2);
      ctx.rotate(angle);
      ctx.drawImage(img, -img.width / 2, -img.height / 2);
      
      // Convert to blob
      canvas.toBlob((blob) => {
        if (blob) {
          if (processedImageUrl) {
            URL.revokeObjectURL(processedImageUrl);
          }
          processedImageUrl = URL.createObjectURL(blob);
        }
        isProcessing = false;
      }, imageFile!.type);
      
    } catch (err) {
      error = `Error: ${err instanceof Error ? err.message : 'Unknown error'}`;
      isProcessing = false;
    }
  }
  
  async function downloadImage() {
    if (!processedImageUrl) return;
    
    successMessage = '';
    error = '';
    
    // Use Tauri save dialog if available
    if (isTauriAvailable && saveFn && writeFileFn) {
      try {
        // Get file extension from original file
        const ext = imageFile?.name.split('.').pop() || 'png';
        const defaultName = `kairoa_rotated_${imageFile?.name || 'image.png'}`;
        
        // Show save dialog
        const filePath = await saveFn({
          defaultPath: defaultName,
          filters: [{
            name: 'Image',
            extensions: [ext]
          }]
        });
        
        if (filePath) {
          // Fetch the blob from the URL
          const response = await fetch(processedImageUrl);
          const blob = await response.blob();
          const arrayBuffer = await blob.arrayBuffer();
          const uint8Array = new Uint8Array(arrayBuffer);
          
          // Write file using Tauri
          await writeFileFn(filePath, uint8Array);
          
          // Show success message
          successMessage = t('imageTools.saveSuccess');
          setTimeout(() => {
            successMessage = '';
          }, 3000);
        }
      } catch (err) {
        error = `${t('imageTools.saveFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`;
      }
    } else {
      // Fallback to browser download
      const a = document.createElement('a');
      a.href = processedImageUrl;
      a.download = `kairoa_rotated_${imageFile?.name || 'image.png'}`;
      a.click();
      
      successMessage = t('imageTools.downloadStarted');
      setTimeout(() => {
        successMessage = '';
      }, 3000);
    }
  }

  async function convertImageFormat() {
    if (!imageFile || !imageUrl) {
      error = t('imageTools.noImageSelected');
      return;
    }
    
    isProcessing = true;
    error = '';
    
    try {
      const img = new Image();
      img.src = imageUrl;
      
      await new Promise((resolve, reject) => {
        img.onload = resolve;
        img.onerror = reject;
      });
      
      const canvas = document.createElement('canvas');
      const ctx = canvas.getContext('2d');
      
      if (!ctx) {
        throw new Error('Failed to get canvas context');
      }
      
      canvas.width = img.width;
      canvas.height = img.height;
      
      // Draw image to canvas
      ctx.drawImage(img, 0, 0);
      
      // Convert to target format
      const mimeType = `image/${targetFormat === 'jpg' ? 'jpeg' : targetFormat}`;
      const quality = targetFormat === 'jpg' || targetFormat === 'webp' ? 0.92 : undefined;
      
      canvas.toBlob((blob) => {
        if (blob) {
          if (convertedImageUrl) {
            URL.revokeObjectURL(convertedImageUrl);
          }
          convertedImageBlob = blob;
          convertedImageUrl = URL.createObjectURL(blob);
        }
        isProcessing = false;
      }, mimeType, quality);
      
    } catch (err) {
      error = `Error: ${err instanceof Error ? err.message : 'Unknown error'}`;
      isProcessing = false;
    }
  }

  async function downloadConvertedImage() {
    if (!convertedImageUrl || !convertedImageBlob) return;
    
    successMessage = '';
    error = '';
    
    // Use Tauri save dialog if available
    if (isTauriAvailable && saveFn && writeFileFn) {
      try {
        const originalName = imageFile?.name.split('.').slice(0, -1).join('.') || 'image';
        const defaultName = `kairoa_${originalName}.${targetFormat}`;
        
        // Show save dialog
        const filePath = await saveFn({
          defaultPath: defaultName,
          filters: [{
            name: 'Image',
            extensions: [targetFormat]
          }]
        });
        
        if (filePath) {
          const arrayBuffer = await convertedImageBlob.arrayBuffer();
          const uint8Array = new Uint8Array(arrayBuffer);
          
          // Write file using Tauri
          await writeFileFn(filePath, uint8Array);
          
          // Show success message
          successMessage = t('imageTools.saveSuccess');
          setTimeout(() => {
            successMessage = '';
          }, 3000);
        }
      } catch (err) {
        error = `${t('imageTools.saveFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`;
      }
    } else {
      // Fallback to browser download
      const a = document.createElement('a');
      a.href = convertedImageUrl;
      a.download = `kairoa_${imageFile?.name.split('.').slice(0, -1).join('.') || 'image'}.${targetFormat}`;
      a.click();
      
      successMessage = t('imageTools.downloadStarted');
      setTimeout(() => {
        successMessage = '';
      }, 3000);
    }
  }
  
  function clearImage() {
    if (imageUrl) {
      URL.revokeObjectURL(imageUrl);
    }
    if (processedImageUrl) {
      URL.revokeObjectURL(processedImageUrl);
    }
    if (convertedImageUrl) {
      URL.revokeObjectURL(convertedImageUrl);
    }
    imageFile = null;
    imageUrl = '';
    processedImageUrl = '';
    convertedImageUrl = '';
    convertedImageBlob = null;
    rotationAngle = 0;
    realTimeRotation = 0;
    error = '';
    successMessage = '';
    
    // Reset scale
    originalWidth = 0;
    originalHeight = 0;
    scaleWidth = 0;
    scaleHeight = 0;
    scalePercentage = 100;
    
    // 重置 input 元素的 value，以便可以再次选择同一张图片
    const input = document.getElementById('image-upload') as HTMLInputElement;
    const inputScale = document.getElementById('image-upload-scale') as HTMLInputElement;
    if (input) {
      input.value = '';
    }
    if (inputScale) {
      inputScale.value = '';
    }
  }
  
  // Scale functions
  function updateScaleFromPercentage() {
    if (originalWidth === 0 || originalHeight === 0) return;
    
    if (maintainAspectRatio) {
      scaleWidth = Math.round((originalWidth * scalePercentage) / 100);
      scaleHeight = Math.round((originalHeight * scalePercentage) / 100);
    } else {
      // If aspect ratio is not maintained, we need to update percentage based on width
      scalePercentage = Math.round((scaleWidth / originalWidth) * 100);
    }
  }
  
  function updateScaleFromDimensions() {
    if (originalWidth === 0 || originalHeight === 0) return;
    
    if (maintainAspectRatio) {
      // Calculate percentage based on width
      scalePercentage = Math.round((scaleWidth / originalWidth) * 100);
      scaleHeight = Math.round((originalHeight * scalePercentage) / 100);
    } else {
      // Calculate percentage based on average
      const widthPercent = (scaleWidth / originalWidth) * 100;
      const heightPercent = (scaleHeight / originalHeight) * 100;
      scalePercentage = Math.round((widthPercent + heightPercent) / 2);
    }
  }
  
  // Watch for scale percentage changes (only when maintainAspectRatio is true)
  $effect(() => {
    if (toolType === 'scale' && imageUrl && maintainAspectRatio && originalWidth > 0 && originalHeight > 0) {
      scaleWidth = Math.round((originalWidth * scalePercentage) / 100);
      scaleHeight = Math.round((originalHeight * scalePercentage) / 100);
    }
  });
  
  async function scaleImage() {
    if (!imageFile || originalWidth === 0 || originalHeight === 0) {
      error = t('imageTools.noImageSelected');
      return;
    }
    
    if (scaleWidth <= 0 || scaleHeight <= 0) {
      error = t('imageTools.scale.invalidDimensions');
      return;
    }
    
    isProcessing = true;
    error = '';
    
    try {
      const img = new Image();
      img.src = imageUrl;
      
      await new Promise((resolve, reject) => {
        img.onload = resolve;
        img.onerror = reject;
      });
      
      const canvas = document.createElement('canvas');
      const ctx = canvas.getContext('2d');
      
      if (!ctx) {
        throw new Error('Failed to get canvas context');
      }
      
      // Set canvas dimensions to scaled size
      canvas.width = scaleWidth;
      canvas.height = scaleHeight;
      
      // Enable image smoothing for better quality
      ctx.imageSmoothingEnabled = true;
      
      // Use high-quality interpolation for upscaling (when scaling up)
      const isUpscaling = scaleWidth > originalWidth || scaleHeight > originalHeight;
      if (isUpscaling) {
        // Use high quality for upscaling
        ctx.imageSmoothingQuality = 'high';
      } else {
        // Use medium quality for downscaling (faster and still good quality)
        ctx.imageSmoothingQuality = 'medium';
      }
      
      // Draw scaled image
      ctx.drawImage(img, 0, 0, scaleWidth, scaleHeight);
      
      // Convert to blob
      canvas.toBlob((blob) => {
        if (blob) {
          if (processedImageUrl) {
            URL.revokeObjectURL(processedImageUrl);
          }
          processedImageUrl = URL.createObjectURL(blob);
        }
        isProcessing = false;
      }, imageFile.type);
      
    } catch (err) {
      error = `Error: ${err instanceof Error ? err.message : 'Unknown error'}`;
      isProcessing = false;
    }
  }
  
  async function downloadScaledImage() {
    if (!processedImageUrl) return;
    
    successMessage = '';
    error = '';
    
    // Use Tauri save dialog if available
    if (isTauriAvailable && saveFn && writeFileFn) {
      try {
        // Get file extension from original file
        const ext = imageFile?.name.split('.').pop() || 'png';
        const defaultName = `kairoa_scaled_${scaleWidth}x${scaleHeight}_${imageFile?.name || 'image.png'}`;
        
        // Show save dialog
        const filePath = await saveFn({
          defaultPath: defaultName,
          filters: [{
            name: 'Image',
            extensions: [ext]
          }]
        });
        
        if (filePath) {
          // Fetch the blob from the URL
          const response = await fetch(processedImageUrl);
          const blob = await response.blob();
          const arrayBuffer = await blob.arrayBuffer();
          const uint8Array = new Uint8Array(arrayBuffer);
          
          // Write file using Tauri
          await writeFileFn(filePath, uint8Array);
          
          // Show success message
          successMessage = t('imageTools.saveSuccess');
          setTimeout(() => {
            successMessage = '';
          }, 3000);
        }
      } catch (err) {
        error = `${t('imageTools.saveFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`;
      }
    } else {
      // Fallback to browser download
      const a = document.createElement('a');
      a.href = processedImageUrl;
      a.download = `kairoa_scaled_${scaleWidth}x${scaleHeight}_${imageFile?.name || 'image.png'}`;
      a.click();
      
      successMessage = t('imageTools.downloadStarted');
      setTimeout(() => {
        successMessage = '';
      }, 3000);
    }
  }
  
  // 在 window 级别添加拖拽事件监听（用于 Tauri 环境）
  $effect(() => {
    if (!browser) return;
    
    // 在 window 级别添加事件监听，确保在 Tauri 环境中也能捕获
    const handleWindowDragOver = (e: DragEvent) => {
      // 检查是否在我们的应用区域内
      const target = e.target as HTMLElement;
      if (target && (target.closest('.flex-1.flex.flex-col.space-y-4') || target.closest('.border-2.border-dashed'))) {
        e.preventDefault();
        e.stopPropagation();
        handleDragOver(e);
      }
    };
    
    const handleWindowDrop = (e: DragEvent) => {
      const target = e.target as HTMLElement;
      if (target && (target.closest('.flex-1.flex.flex-col.space-y-4') || target.closest('.border-2.border-dashed'))) {
        e.preventDefault();
        e.stopPropagation();
        handleDrop(e);
      }
    };
    
    const handleWindowDragEnter = (e: DragEvent) => {
      const target = e.target as HTMLElement;
      if (target && (target.closest('.flex-1.flex.flex-col.space-y-4') || target.closest('.border-2.border-dashed'))) {
        e.preventDefault();
        e.stopPropagation();
        handleDragEnter(e);
      }
    };
    
    window.addEventListener('dragover', handleWindowDragOver, true);
    window.addEventListener('drop', handleWindowDrop, true);
    window.addEventListener('dragenter', handleWindowDragEnter, true);
    
    return () => {
      window.removeEventListener('dragover', handleWindowDragOver, true);
      window.removeEventListener('drop', handleWindowDrop, true);
      window.removeEventListener('dragenter', handleWindowDragEnter, true);
      if (imageUrl) URL.revokeObjectURL(imageUrl);
      if (processedImageUrl) URL.revokeObjectURL(processedImageUrl);
    };
  });
</script>

<div class="flex flex-col h-full p-2">
  <div class="flex-1 flex flex-col min-h-0 card">
    <div class="flex-shrink-0 mb-4">
      <!-- Tool Type Tabs -->
      <div class="border-b border-gray-200 dark:border-gray-700">
        <div class="flex gap-6">
          <button
            onclick={() => switchToolType('rotate')}
            class="px-4 py-2 relative transition-colors font-medium {toolType === 'rotate'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            {t('imageTools.rotate.title')}
            {#if toolType === 'rotate'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:text-primary-400"></span>
            {/if}
          </button>
          <button
            onclick={() => switchToolType('scale')}
            class="px-4 py-2 relative transition-colors font-medium {toolType === 'scale'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            {t('imageTools.scale.title')}
            {#if toolType === 'scale'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:text-primary-400"></span>
            {/if}
          </button>
          <button
            onclick={() => switchToolType('convert')}
            class="px-4 py-2 relative transition-colors font-medium {toolType === 'convert'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            {t('imageTools.convert.title')}
            {#if toolType === 'convert'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:text-primary-400"></span>
            {/if}
          </button>
        </div>
      </div>
    </div>

    <!-- Rotate Tool -->
    {#if toolType === 'rotate'}
      <div 
        class="flex-1 flex flex-col space-y-4 min-h-0 overflow-y-auto transition-all duration-200 {isDragging ? 'ring-2 ring-primary-500 dark:ring-primary-400 ring-offset-2 bg-primary-50/50 dark:bg-primary-900/20' : ''}"
        ondrop={(e) => { e.stopPropagation(); handleDrop(e); }}
        ondragover={(e) => { e.stopPropagation(); handleDragOver(e); }}
        ondragenter={(e) => { e.stopPropagation(); handleDragEnter(e); }}
        ondragleave={(e) => { e.stopPropagation(); handleDragLeave(e); }}
      >
        <!-- File Upload Input - 统一放在这里 -->
        <input
          type="file"
          id="image-upload"
          accept="image/*"
          onchange={handleFileSelect}
          class="hidden"
        />
        
        <!-- File Upload - 只在没有图片时显示 -->
        {#if !imageUrl}
          <div class="flex-1 flex flex-col items-center justify-center border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg hover:border-primary-500 dark:hover:border-primary-400 transition-colors bg-gray-50 dark:bg-gray-800/50 {isDragging ? 'border-primary-500 dark:border-primary-400 bg-primary-50 dark:bg-primary-900/30' : ''}">
            <div
              role="button"
              tabindex="0"
              class="text-center cursor-pointer"
              ondrop={handleDrop}
              ondragover={handleDragOver}
              ondragenter={handleDragEnter}
              ondragleave={handleDragLeave}
              onclick={() => document.getElementById('image-upload')?.click()}
              onkeydown={(e) => e.key === 'Enter' && document.getElementById('image-upload')?.click()}
            >
              <Upload class="w-12 h-12 text-gray-400 dark:text-gray-500 mb-4 mx-auto" />
              <p class="text-gray-600 dark:text-gray-400 mb-2">
                {t('imageTools.dragDropImage')}
              </p>
              <p class="text-sm text-gray-500 dark:text-gray-500">
                {t('imageTools.supportedFormats')}
              </p>
            </div>
          </div>
        {/if}

        {#if imageUrl}
          <!-- Rotation Controls -->
          <div class="flex-shrink-0">
            <div class="relative">
              <!-- 滑动条和输入框容器 -->
              <div class="flex items-center gap-4">
                <!-- 滑动条容器 -->
                <div class="relative flex-1 py-8">
                  <input
                    type="range"
                    id="rotation-angle"
                    bind:value={rotationAngle}
                    min="0"
                    max="360"
                    step="1"
                    class="w-full appearance-none h-2 bg-gray-200 dark:bg-gray-700 rounded-lg outline-none cursor-pointer"
                  />
                  <!-- 美化滑动条轨道 -->
                  <style>
                    #rotation-angle::-webkit-slider-thumb {
                      appearance: none;
                      width: 20px;
                      height: 20px;
                      border-radius: 50%;
                      background: #818089;
                      cursor: pointer;
                      border: 2px solid white;
                      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
                      transition: all 0.2s ease;
                    }
                    #rotation-angle::-webkit-slider-thumb:hover {
                      background: #605f69;
                      transform: scale(1.1);
                    }
                    #rotation-angle::-moz-range-thumb {
                      width: 20px;
                      height: 20px;
                      border-radius: 50%;
                      background: #818089;
                      cursor: pointer;
                      border: 2px solid white;
                      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
                      transition: all 0.2s ease;
                    }
                    #rotation-angle::-moz-range-thumb:hover {
                      background: #605f69;
                      transform: scale(1.1);
                    }
                  </style>
                  
                  <!-- 快捷角度标记 -->
                  <div class="absolute left-0 right-0 top-0 pointer-events-none" style="top: 0.5rem;">
                    <!-- 90度标记 -->
                    <button
                      onclick={(e) => { e.stopPropagation(); rotationAngle = 90; }}
                      style="left: 25%;"
                      class="absolute -translate-x-1/2 top-0 w-6 h-10 flex flex-col items-center justify-end cursor-pointer group pointer-events-auto"
                      aria-label="Rotate 90 degrees"
                    >
                      <div 
                        class={`w-3 h-3 rounded-full mb-1 transition-all duration-200 ease ${rotationAngle === 90 ? 'bg-primary-600 dark:bg-primary-400 scale-125' : 'bg-gray-300 dark:bg-gray-600 group-hover:bg-primary-300 dark:group-hover:bg-primary-700'}`}
                      ></div>
                      <div 
                        class={`w-0.5 h-6 transition-all duration-200 ease ${rotationAngle === 90 ? 'bg-primary-600 dark:bg-primary-400' : 'bg-gray-300 dark:bg-gray-600 group-hover:bg-primary-300 dark:group-hover:bg-primary-700'}`}
                      ></div>
                      <span class={`absolute top-12 text-xs whitespace-nowrap transition-colors duration-200 ${rotationAngle === 90 ? 'text-primary-600 dark:text-primary-400 font-medium' : 'text-gray-500 dark:text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-400'}`}>
                        90°
                      </span>
                    </button>
                    
                    <!-- 180度标记 -->
                    <button
                      onclick={(e) => { e.stopPropagation(); rotationAngle = 180; }}
                      style="left: 50%;"
                      class="absolute -translate-x-1/2 top-0 w-6 h-10 flex flex-col items-center justify-end cursor-pointer group pointer-events-auto"
                      aria-label="Rotate 180 degrees"
                    >
                      <div 
                        class={`w-3 h-3 rounded-full mb-1 transition-all duration-200 ease ${rotationAngle === 180 ? 'bg-primary-600 dark:bg-primary-400 scale-125' : 'bg-gray-300 dark:bg-gray-600 group-hover:bg-primary-300 dark:group-hover:bg-primary-700'}`}
                      ></div>
                      <div 
                        class={`w-0.5 h-6 transition-all duration-200 ease ${rotationAngle === 180 ? 'bg-primary-600 dark:bg-primary-400' : 'bg-gray-300 dark:bg-gray-600 group-hover:bg-primary-300 dark:group-hover:bg-primary-700'}`}
                      ></div>
                      <span class={`absolute top-12 text-xs whitespace-nowrap transition-colors duration-200 ${rotationAngle === 180 ? 'text-primary-600 dark:text-primary-400 font-medium' : 'text-gray-500 dark:text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-400'}`}>
                        180°
                      </span>
                    </button>
                    
                    <!-- 270度标记 -->
                    <button
                      onclick={(e) => { e.stopPropagation(); rotationAngle = 270; }}
                      style="left: 75%;"
                      class="absolute -translate-x-1/2 top-0 w-6 h-10 flex flex-col items-center justify-end cursor-pointer group pointer-events-auto"
                      aria-label="Rotate 270 degrees"
                    >
                      <div 
                        class={`w-3 h-3 rounded-full mb-1 transition-all duration-200 ease ${rotationAngle === 270 ? 'bg-primary-600 dark:bg-primary-400 scale-125' : 'bg-gray-300 dark:bg-gray-600 group-hover:bg-primary-300 dark:group-hover:bg-primary-700'}`}
                      ></div>
                      <div 
                        class={`w-0.5 h-6 transition-all duration-200 ease ${rotationAngle === 270 ? 'bg-primary-600 dark:bg-primary-400' : 'bg-gray-300 dark:bg-gray-600 group-hover:bg-primary-300 dark:group-hover:bg-primary-700'}`}
                      ></div>
                      <span class={`absolute top-12 text-xs whitespace-nowrap transition-colors duration-200 ${rotationAngle === 270 ? 'text-primary-600 dark:text-primary-400 font-medium' : 'text-gray-500 dark:text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-400'}`}>
                        270°
                      </span>
                    </button>
                  </div>
                </div>
                
                <!-- 角度数值输入 -->
                <div class="flex items-center gap-2 flex-shrink-0">
                  <input
                    type="number"
                    bind:value={rotationAngle}
                    min="0"
                    max="360"
                    class="input w-24 text-center"
                  />
                  <span class="text-gray-600 dark:text-gray-400">°</span>
                </div>
                
                <!-- Action Buttons -->
                <div class="flex items-center gap-2 flex-shrink-0">
                  <button
                    onclick={rotateImage}
                    disabled={isProcessing}
                    class="px-4 py-2 text-white rounded-lg transition-colors font-medium hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                    style="background-color: #818089;"
                  >
                    <RotateCw class="w-4 h-4" />
                    {isProcessing ? t('imageTools.processing') : t('imageTools.rotate.apply')}
                  </button>
                  {#if processedImageUrl}
                    <button
                      onclick={downloadImage}
                      class="btn-secondary flex items-center gap-2"
                    >
                      <Download class="w-4 h-4" />
                      {t('imageTools.download')}
                    </button>
                  {/if}
                  <button
                    onclick={clearImage}
                    disabled={isProcessing}
                    class="btn-secondary"
                  >
                    {t('common.clear')}
                  </button>
                </div>
              </div>
            </div>
          </div>

          {#if error}
            <div class="flex-shrink-0 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-3">
              <p class="text-sm text-red-800 dark:text-red-200">
                {error}
              </p>
            </div>
          {/if}

          {#if successMessage}
            <div class="flex-shrink-0 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-3">
              <p class="text-sm text-green-800 dark:text-green-200">
                {successMessage}
              </p>
            </div>
          {/if}

          <!-- Image Preview -->
          <div class="flex-1 grid grid-cols-1 md:grid-cols-2 gap-4 min-h-0">
            <div class="flex flex-col">
              <h3 class="text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('imageTools.original')}
              </h3>
              <div 
                class="flex-1 border rounded-lg overflow-hidden flex items-center justify-center transition-all duration-200 {isDragging ? 'border-primary-500 dark:border-primary-400 bg-primary-50 dark:bg-primary-900/30 border-2' : 'border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-800'}"
                ondrop={handleDrop}
                ondragover={handleDragOver}
                ondragenter={handleDragEnter}
                ondragleave={handleDragLeave}
              >
                <img src={imageUrl} alt="Original" class="max-w-full max-h-full object-contain pointer-events-none" />
              </div>
            </div>
            <div class="flex flex-col">
              <h3 class="text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('imageTools.processed')}
              </h3>
              <div 
                class="flex-1 border rounded-lg overflow-hidden flex items-center justify-center transition-all duration-200 {isDragging ? 'border-primary-500 dark:border-primary-400 bg-primary-50 dark:bg-primary-900/30 border-2' : 'border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-800'}"
                ondrop={handleDrop}
                ondragover={handleDragOver}
                ondragenter={handleDragEnter}
                ondragleave={handleDragLeave}
              >
                <div 
                  class="relative w-full h-full flex items-center justify-center"
                  style={`transform: rotate(${realTimeRotation}deg); transition: transform 0.1s ease;`}
                >
                  <img src={imageUrl} alt="Rotating Preview" class="max-w-full max-h-full object-contain pointer-events-none" />
                </div>
              </div>
              {#if processedImageUrl}
                <div class="mt-2 text-sm text-gray-500 dark:text-gray-400">
                  已生成处理后的图片，可点击下载按钮保存
                </div>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Scale Tool -->
    {#if toolType === 'scale'}
      <div 
        class="flex-1 flex flex-col space-y-4 min-h-0 overflow-y-auto transition-all duration-200 {isDragging ? 'ring-2 ring-primary-500 dark:ring-primary-400 ring-offset-2 bg-primary-50/50 dark:bg-primary-900/20' : ''}"
        ondrop={(e) => { e.stopPropagation(); handleDrop(e); }}
        ondragover={(e) => { e.stopPropagation(); handleDragOver(e); }}
        ondragenter={(e) => { e.stopPropagation(); handleDragEnter(e); }}
        ondragleave={(e) => { e.stopPropagation(); handleDragLeave(e); }}
      >
        <!-- File Upload Input - 统一放在这里 -->
        <input
          type="file"
          id="image-upload-scale"
          accept="image/*"
          onchange={handleFileSelect}
          class="hidden"
        />
        
        <!-- File Upload - 只在没有图片时显示 -->
        {#if !imageUrl}
          <div class="flex-1 flex flex-col items-center justify-center border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg hover:border-primary-500 dark:hover:border-primary-400 transition-colors bg-gray-50 dark:bg-gray-800/50 {isDragging ? 'border-primary-500 dark:border-primary-400 bg-primary-50 dark:bg-primary-900/30' : ''}">
            <div
              role="button"
              tabindex="0"
              class="text-center cursor-pointer"
              ondrop={handleDrop}
              ondragover={handleDragOver}
              ondragenter={handleDragEnter}
              ondragleave={handleDragLeave}
              onclick={() => document.getElementById('image-upload-scale')?.click()}
              onkeydown={(e) => e.key === 'Enter' && document.getElementById('image-upload-scale')?.click()}
            >
              <Upload class="w-12 h-12 text-gray-400 dark:text-gray-500 mb-4 mx-auto" />
              <p class="text-gray-600 dark:text-gray-400 mb-2">
                {t('imageTools.dragDropImage')}
              </p>
              <p class="text-sm text-gray-500 dark:text-gray-500">
                {t('imageTools.supportedFormats')}
              </p>
            </div>
          </div>
        {/if}

        {#if imageUrl}
          <!-- Scale Controls -->
          <div class="flex-shrink-0 space-y-4">
            <!-- Scale Percentage Slider -->
            <div class="relative">
              <div class="flex items-center gap-4">
                <!-- Slider -->
                <div class="relative flex-1 py-8">
                  <input
                    type="range"
                    id="scale-percentage"
                    bind:value={scalePercentage}
                    min="1"
                    max="500"
                    step="1"
                    class="w-full appearance-none h-2 bg-gray-200 dark:bg-gray-700 rounded-lg outline-none cursor-pointer"
                    oninput={() => updateScaleFromPercentage()}
                  />
                  <style>
                    #scale-percentage::-webkit-slider-thumb {
                      appearance: none;
                      width: 20px;
                      height: 20px;
                      border-radius: 50%;
                      background: #818089;
                      cursor: pointer;
                      border: 2px solid white;
                      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
                      transition: all 0.2s ease;
                    }
                    #scale-percentage::-webkit-slider-thumb:hover {
                      background: #605f69;
                      transform: scale(1.1);
                    }
                    #scale-percentage::-moz-range-thumb {
                      width: 20px;
                      height: 20px;
                      border-radius: 50%;
                      background: #818089;
                      cursor: pointer;
                      border: 2px solid white;
                      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
                      transition: all 0.2s ease;
                    }
                    #scale-percentage::-moz-range-thumb:hover {
                      background: #605f69;
                      transform: scale(1.1);
                    }
                  </style>
                  
                  <!-- Original Size Marker (100%) -->
                  <div class="absolute left-0 right-0 top-0 pointer-events-none" style="top: 0.5rem;">
                    <button
                      onclick={(e) => { e.stopPropagation(); scalePercentage = 100; updateScaleFromPercentage(); }}
                      style="left: calc((100 - 1) / (500 - 1) * 100%);"
                      class="absolute -translate-x-1/2 top-0 w-6 h-10 flex flex-col items-center justify-end cursor-pointer group pointer-events-auto"
                      aria-label="Original size (100%)"
                    >
                      <div 
                        class={`w-3 h-3 rounded-full mb-1 transition-all duration-200 ease ${scalePercentage === 100 ? 'bg-primary-600 dark:bg-primary-400 scale-125' : 'bg-gray-300 dark:bg-gray-600 group-hover:bg-primary-300 dark:group-hover:bg-primary-700'}`}
                      ></div>
                      <div 
                        class={`w-0.5 h-6 transition-all duration-200 ease ${scalePercentage === 100 ? 'bg-primary-600 dark:bg-primary-400' : 'bg-gray-300 dark:bg-gray-600 group-hover:bg-primary-300 dark:group-hover:bg-primary-700'}`}
                      ></div>
                      <span class={`absolute top-12 text-xs whitespace-nowrap transition-colors duration-200 ${scalePercentage === 100 ? 'text-primary-600 dark:text-primary-400 font-medium' : 'text-gray-500 dark:text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-400'}`}>
                        100%
                      </span>
                    </button>
                  </div>
                </div>
                
                <!-- Percentage Input -->
                <div class="flex items-center gap-2 flex-shrink-0">
                  <input
                    type="number"
                    bind:value={scalePercentage}
                    min="1"
                    max="500"
                    class="input w-24 text-center"
                    oninput={() => updateScaleFromPercentage()}
                  />
                  <span class="text-gray-600 dark:text-gray-400">%</span>
                </div>
                
                <!-- Action Buttons -->
                <div class="flex items-center gap-2 flex-shrink-0">
                  <button
                    onclick={scaleImage}
                    disabled={isProcessing}
                    class="px-4 py-2 text-white rounded-lg transition-colors font-medium hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                    style="background-color: #818089;"
                  >
                    <RotateCw class="w-4 h-4" />
                    {isProcessing ? t('imageTools.processing') : t('imageTools.scale.apply')}
                  </button>
                  {#if processedImageUrl}
                    <button
                      onclick={downloadScaledImage}
                      class="btn-secondary flex items-center gap-2"
                    >
                      <Download class="w-4 h-4" />
                      {t('imageTools.download')}
                    </button>
                  {/if}
                  <button
                    onclick={clearImage}
                    disabled={isProcessing}
                    class="btn-secondary"
                  >
                    {t('common.clear')}
                  </button>
                </div>
              </div>
            </div>
            
            <!-- Dimensions Input -->
            <div class="flex items-center gap-4">
              <div class="flex items-center gap-2">
                <label class="text-sm text-gray-600 dark:text-gray-400">{t('imageTools.scale.width')}:</label>
                <input
                  type="number"
                  bind:value={scaleWidth}
                  min="1"
                  class="input w-24 text-center"
                  oninput={() => updateScaleFromDimensions()}
                  disabled={maintainAspectRatio}
                />
                <span class="text-gray-600 dark:text-gray-400">px</span>
              </div>
              <div class="flex items-center gap-2">
                <label class="text-sm text-gray-600 dark:text-gray-400">{t('imageTools.scale.height')}:</label>
                <input
                  type="number"
                  bind:value={scaleHeight}
                  min="1"
                  class="input w-24 text-center"
                  oninput={() => updateScaleFromDimensions()}
                  disabled={maintainAspectRatio}
                />
                <span class="text-gray-600 dark:text-gray-400">px</span>
              </div>
              <div class="flex items-center gap-2">
                <input
                  type="checkbox"
                  bind:checked={maintainAspectRatio}
                  id="maintain-aspect-ratio"
                  class="w-4 h-4 rounded border-gray-300 dark:border-gray-600"
                />
                <label for="maintain-aspect-ratio" class="text-sm text-gray-600 dark:text-gray-400">
                  {t('imageTools.scale.maintainAspectRatio')}
                </label>
              </div>
            </div>
            
            {#if originalWidth > 0 && originalHeight > 0}
              <div class="text-xs text-gray-500 dark:text-gray-400">
                {t('imageTools.scale.originalSize')}: {originalWidth} × {originalHeight} px
              </div>
            {/if}
          </div>

          <!-- Image Preview -->
          <div class="flex-1 grid grid-cols-1 md:grid-cols-2 gap-4 min-h-0">
            <div class="flex flex-col">
              <h3 class="text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('imageTools.original')}
              </h3>
              <div 
                class="flex-1 border rounded-lg overflow-hidden flex items-center justify-center transition-all duration-200 {isDragging ? 'border-primary-500 dark:border-primary-400 bg-primary-50 dark:bg-primary-900/30 border-2' : 'border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-800'}"
                ondrop={handleDrop}
                ondragover={handleDragOver}
                ondragenter={handleDragEnter}
                ondragleave={handleDragLeave}
              >
                <img src={imageUrl} alt="Original" class="max-w-full max-h-full object-contain pointer-events-none" />
              </div>
            </div>
            {#if processedImageUrl}
              <div class="flex flex-col">
                <h3 class="text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('imageTools.processed')}
                </h3>
                <div 
                  class="flex-1 border rounded-lg overflow-hidden flex items-center justify-center transition-all duration-200 {isDragging ? 'border-primary-500 dark:border-primary-400 bg-primary-50 dark:bg-primary-900/30 border-2' : 'border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-800'}"
                  ondrop={handleDrop}
                  ondragover={handleDragOver}
                  ondragenter={handleDragEnter}
                  ondragleave={handleDragLeave}
                >
                  <img src={processedImageUrl} alt="Scaled" class="max-w-full max-h-full object-contain pointer-events-none" />
                </div>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Convert Tool -->
    {#if toolType === 'convert'}
      <div 
        class="flex-1 flex flex-col space-y-4 min-h-0 overflow-y-auto transition-all duration-200 {isDragging ? 'ring-2 ring-primary-500 dark:ring-primary-400 ring-offset-2 bg-primary-50/50 dark:bg-primary-900/20' : ''}"
        ondrop={(e) => { e.stopPropagation(); handleDrop(e); }}
        ondragover={(e) => { e.stopPropagation(); handleDragOver(e); }}
        ondragenter={(e) => { e.stopPropagation(); handleDragEnter(e); }}
        ondragleave={(e) => { e.stopPropagation(); handleDragLeave(e); }}
      >
        <!-- File Upload Input -->
        <input
          type="file"
          id="image-upload-convert"
          accept="image/*"
          onchange={handleFileSelect}
          class="hidden"
        />
        
        <!-- 图片上传区域 -->
        <div class="flex-1 flex flex-col items-center justify-center border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-8 hover:border-primary-500 dark:hover:border-primary-400 transition-colors bg-gray-50 dark:bg-gray-800/50">
          {#if !imageUrl}
            <div
              role="button"
              tabindex="0"
              class="text-center cursor-pointer"
              onclick={() => document.getElementById('image-upload-convert')?.click()}
              onkeydown={(e) => e.key === 'Enter' && document.getElementById('image-upload-convert')?.click()}
            >
              <Upload class="w-12 h-12 text-gray-400 dark:text-gray-500 mb-4 mx-auto" />
              <p class="text-gray-600 dark:text-gray-400 mb-2">
                {t('imageTools.dragDropImage')}
              </p>
              <p class="text-sm text-gray-500 dark:text-gray-500">
                {t('imageTools.supportedFormats')}
              </p>
            </div>
          {:else}
            <div class="w-full max-w-4xl grid grid-cols-1 md:grid-cols-2 gap-6">
              <!-- 原始图片 -->
              <div class="space-y-3 flex flex-col">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
                  {t('imageTools.original')}
                </h3>
                <div class="border-2 border-gray-300 dark:border-gray-600 rounded-lg overflow-hidden bg-gray-100 dark:bg-gray-900 flex items-center justify-center" style="min-height: 384px;">
                  <img
                    src={imageUrl}
                    alt="Original"
                    class="w-full h-auto max-h-96 object-contain"
                  />
                </div>
                <p class="text-sm text-gray-600 dark:text-gray-400">
                  {imageFile?.name || 'image'}
                </p>
              </div>

              <!-- 转换后图片 -->
              <div class="space-y-3 flex flex-col">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
                  {t('imageTools.convert.converted')}
                </h3>
                <div class="border-2 {convertedImageUrl ? 'border-primary-500 dark:border-primary-400' : 'border-dashed border-gray-300 dark:border-gray-600'} rounded-lg overflow-hidden {convertedImageUrl ? 'bg-gray-100 dark:bg-gray-900' : 'bg-gray-50 dark:bg-gray-800/50'} flex items-center justify-center" style="min-height: 384px;">
                  {#if convertedImageUrl}
                    <img
                      src={convertedImageUrl}
                      alt="Converted"
                      class="w-full h-auto max-h-96 object-contain"
                    />
                  {:else}
                    <p class="text-gray-500 dark:text-gray-400">
                      {t('imageTools.convert.previewPlaceholder')}
                    </p>
                  {/if}
                </div>
                {#if convertedImageUrl}
                  <p class="text-sm text-gray-600 dark:text-gray-400">
                    {imageFile?.name.split('.').slice(0, -1).join('.') || 'image'}.{targetFormat}
                  </p>
                {/if}
              </div>
            </div>

            <!-- 格式选择和控制 -->
            <div class="w-full max-w-4xl space-y-4 mt-6">
              <div class="flex items-center gap-4">
                <label class="text-sm font-medium text-gray-700 dark:text-gray-300">
                  {t('imageTools.convert.targetFormat')}:
                </label>
                <select
                  bind:value={targetFormat}
                  class="input"
                  onchange={() => {
                    if (convertedImageUrl) {
                      convertImageFormat();
                    }
                  }}
                >
                  <option value="png">PNG</option>
                  <option value="jpg">JPG</option>
                  <option value="webp">WebP</option>
                  <option value="gif">GIF</option>
                </select>
              </div>

              <div class="flex gap-3">
                <button
                  onclick={convertImageFormat}
                  disabled={isProcessing || !imageUrl}
                  class="btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {#if isProcessing}
                    {t('imageTools.processing')}
                  {:else}
                    {t('imageTools.convert.convert')}
                  {/if}
                </button>
                {#if convertedImageUrl}
                  <button
                    onclick={downloadConvertedImage}
                    class="btn-secondary flex items-center gap-2"
                  >
                    <Download class="w-4 h-4" />
                    {t('imageTools.download')}
                  </button>
                {/if}
                <button
                  onclick={clearImage}
                  class="btn-secondary"
                >
                  {t('imageTools.clear')}
                </button>
              </div>
            </div>
          {/if}
        </div>

        {#if error}
          <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
            <p class="text-sm text-red-800 dark:text-red-200">{error}</p>
          </div>
        {/if}

        {#if successMessage}
          <div class="p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
            <p class="text-sm text-green-800 dark:text-green-200">{successMessage}</p>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
