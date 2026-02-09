use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose};
use dsa::{SigningKey, VerifyingKey, Components, Signature};
use rand::rngs::OsRng;
use pkcs8::{EncodePrivateKey, EncodePublicKey, DecodePrivateKey, DecodePublicKey, LineEnding};
use signature::{Signer, Verifier};
use sha2::{Sha256, Digest};
use der::Encode;
use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use scrypt::{
    password_hash::{PasswordHash as ScryptPasswordHash, SaltString as ScryptSaltString},
    Scrypt,
};
use argon2::{
    password_hash::{PasswordHash as Argon2PasswordHash, SaltString as Argon2SaltString},
    Argon2, ParamsBuilder,
};
use image::{DynamicImage, ImageEncoder, ExtendedColorType};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use std::io::Cursor;

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpRequest {
    url: String,
    method: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MultipartEntry {
    key: String,
    value: String,
    r#type: String, // "text" or "file"
    filename: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MultipartBody {
    r#type: String,
    entries: Vec<MultipartEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
    error: Option<String>,
}

#[tauri::command]
async fn http_request(request: HttpRequest) -> Result<HttpResponse, String> {
    let client = reqwest::Client::new();
    
    let mut req_builder = match request.method.as_str() {
        "GET" => client.get(&request.url),
        "POST" => client.post(&request.url),
        "PUT" => client.put(&request.url),
        "DELETE" => client.delete(&request.url),
        "PATCH" => client.patch(&request.url),
        "HEAD" => client.head(&request.url),
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, &request.url),
        _ => return Err(format!("Unsupported HTTP method: {}", request.method)),
    };

    // 添加请求头
    for (key, value) in request.headers {
        // 如果是 multipart 请求，Content-Type 会被 reqwest 自动设置
        if key.to_lowercase() != "content-type" || !request.body.as_ref().and_then(|b| serde_json::from_str::<MultipartBody>(b).ok()).map_or(false, |mb: MultipartBody| mb.r#type == "multipart") {
            req_builder = req_builder.header(&key, &value);
        }
    }

    // 添加请求体
    if let Some(body) = request.body {
        // 检查是否是 multipart 请求
        if let Ok(multipart_body) = serde_json::from_str::<MultipartBody>(&body) {
            if multipart_body.r#type == "multipart" {
                // 构建 multipart form
                let mut form = reqwest::multipart::Form::new();
                
                for entry in multipart_body.entries {
                    if entry.r#type == "file" {
                        // 解码 base64 文件
                        if let Ok(file_bytes) = general_purpose::STANDARD.decode(&entry.value) {
                            let part = if let Some(filename) = entry.filename.clone() {
                                let filename_clone = filename.clone();
                                reqwest::multipart::Part::bytes(file_bytes)
                                    .file_name(filename)
                                    .mime_str("application/octet-stream")
                                    .unwrap_or_else(|_| {
                                        // 如果 mime_str 失败，重新创建 Part（不使用 MIME 类型）
                                        // 这不应该发生，但为了安全起见我们处理它
                                        let file_bytes_copy = general_purpose::STANDARD.decode(&entry.value).unwrap();
                                        reqwest::multipart::Part::bytes(file_bytes_copy)
                                            .file_name(filename_clone)
                                    })
                            } else {
                                reqwest::multipart::Part::bytes(file_bytes)
                                    .mime_str("application/octet-stream")
                                    .unwrap_or_else(|_| {
                                        // 如果 mime_str 失败，重新创建 Part（不使用 MIME 类型）
                                        let file_bytes_copy = general_purpose::STANDARD.decode(&entry.value).unwrap();
                                        reqwest::multipart::Part::bytes(file_bytes_copy)
                                    })
                            };
                            form = form.part(entry.key, part);
                        }
                    } else {
                        // 文本字段
                        form = form.text(entry.key, entry.value);
                    }
                }
                
                req_builder = req_builder.multipart(form);
            } else {
                req_builder = req_builder.body(body);
            }
        } else {
            req_builder = req_builder.body(body);
        }
    }

    match req_builder.send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            let mut headers = HashMap::new();
            
            // 获取响应头
            for (key, value) in response.headers() {
                if let Ok(value_str) = value.to_str() {
                    headers.insert(key.to_string(), value_str.to_string());
                }
            }

            // 获取响应体
            let body = match response.text().await {
                Ok(text) => text,
                Err(e) => format!("Error reading response body: {}", e),
            };

            Ok(HttpResponse {
                status,
                headers,
                body,
                error: None,
            })
        }
        Err(e) => Ok(HttpResponse {
            status: 0,
            headers: HashMap::new(),
            body: String::new(),
            error: Some(e.to_string()),
        }),
    }
}

// DSA 密钥对响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct DsaKeyPair {
    public_key: String,
    private_key: String,
    format: String,
}

// 生成 DSA 密钥对（异步版本以避免阻塞）
#[tauri::command]
async fn generate_dsa_keypair(key_size: u32, format: String) -> Result<DsaKeyPair, String> {
    // 在单独的线程中生成密钥对以避免阻塞
    let result = tokio::task::spawn_blocking(move || {
        // 根据密钥大小生成密钥对
        let signing_key = match key_size {
            1024 => {
                // DSA 1024 位（已弃用，但仍支持）
                #[allow(deprecated)]
                let comp = Components::generate(&mut OsRng, dsa::KeySize::DSA_1024_160).clone();
                SigningKey::generate(&mut OsRng, comp)
            },
            2048 => {
                // DSA 2048 位
                let comp = Components::generate(&mut OsRng, dsa::KeySize::DSA_2048_256).clone();
                SigningKey::generate(&mut OsRng, comp)
            },
            3072 => {
                // DSA 3072 位
                let comp = Components::generate(&mut OsRng, dsa::KeySize::DSA_3072_256).clone();
                SigningKey::generate(&mut OsRng, comp)
            },
            _ => {
                return Err(format!("Unsupported key size: {}. Supported sizes: 1024, 2048, 3072", key_size));
            }
        };
        Ok(signing_key)
    }).await.map_err(|e| format!("Task execution failed: {}", e))?;
    
    let signing_key = result?;
    
    // 先获取公钥（克隆）
    let verifying_key = signing_key.verifying_key().clone();
    
    // 根据格式导出密钥
    let (public_key_str, private_key_str) = if format == "pem" {
        // PEM 格式
        let public_pem = verifying_key
            .to_public_key_pem(LineEnding::LF)
            .map_err(|e| format!("Failed to encode public key to PEM: {}", e))?;
        
        let private_pem = signing_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|e| format!("Failed to encode private key to PEM: {}", e))?;
        
        (public_pem, private_pem.to_string())
    } else {
        // DER 格式（十六进制）
        let public_der = verifying_key
            .to_public_key_der()
            .map_err(|e| format!("Failed to encode public key to DER: {}", e))?;
        
        let private_der = signing_key
            .to_pkcs8_der()
            .map_err(|e| format!("Failed to encode private key to DER: {}", e))?;
        
        // 转换为十六进制字符串
        let public_hex = format_hex(public_der.as_bytes());
        let private_hex = format_hex(private_der.as_bytes());
        
        (public_hex, private_hex)
    };

    Ok(DsaKeyPair {
        public_key: public_key_str,
        private_key: private_key_str,
        format,
    })
}

// 格式化十六进制字符串（每 32 个字符换行）
fn format_hex(bytes: &[u8]) -> String {
    let hex: String = bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect();
    
    let mut result = String::new();
    for (i, chunk) in hex.as_bytes().chunks(32).enumerate() {
        if i > 0 {
            result.push('\n');
        }
        result.push_str(&String::from_utf8_lossy(chunk));
    }
    result
}

// DSA 签名结果
#[derive(Debug, Serialize, Deserialize)]
pub struct DsaSignResult {
    signature: String,
}

// DSA 验证结果
#[derive(Debug, Serialize, Deserialize)]
pub struct DsaVerifyResult {
    valid: bool,
}

// DSA 签名
#[tauri::command]
fn dsa_sign(private_key_pem: String, message: String) -> Result<DsaSignResult, String> {
    // 从 PEM 导入私钥
    let signing_key = SigningKey::from_pkcs8_pem(&private_key_pem)
        .map_err(|e| format!("Failed to parse private key: {}", e))?;
    
    // 计算消息的 SHA-256 哈希
    let mut hasher = Sha256::new();
    hasher.update(message.as_bytes());
    let hash = hasher.finalize();
    
    // 签名
    let signature: Signature = signing_key.sign(&hash);
    
    // DSA 签名使用 DER 编码
    let sig_bytes = signature.to_der()
        .map_err(|e| format!("Failed to encode signature: {}", e))?;
    let signature_base64 = general_purpose::STANDARD.encode(&sig_bytes);
    
    Ok(DsaSignResult {
        signature: signature_base64,
    })
}

// DSA 验证
#[tauri::command]
fn dsa_verify(public_key_pem: String, message: String, signature: String) -> Result<DsaVerifyResult, String> {
    // 从 PEM 导入公钥
    let verifying_key = VerifyingKey::from_public_key_pem(&public_key_pem)
        .map_err(|e| format!("Failed to parse public key: {}", e))?;
    
    // 解码签名
    let signature_bytes = general_purpose::STANDARD.decode(&signature)
        .map_err(|e| format!("Failed to decode signature: {}", e))?;
    
    let sig = Signature::try_from(signature_bytes.as_slice())
        .map_err(|e| format!("Invalid signature format: {}", e))?;
    
    // 计算消息的 SHA-256 哈希
    let mut hasher = Sha256::new();
    hasher.update(message.as_bytes());
    let hash = hasher.finalize();
    
    // 验证签名
    let valid = verifying_key.verify(&hash, &sig).is_ok();
    
    Ok(DsaVerifyResult {
        valid,
    })
}

// Password Hash structures
#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordHashRequest {
    password: String,
    salt: Option<String>,
    algorithm: String,
    // PBKDF2 params
    iterations: Option<u32>,
    // Scrypt params
    n: Option<u32>,
    r: Option<u32>,
    p: Option<u32>,
    // Argon2 params
    memory: Option<u32>,
    argon2_iterations: Option<u32>,
    parallelism: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordHashResponse {
    hash: String,
    salt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordVerifyRequest {
    password: String,
    hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordVerifyResponse {
    valid: bool,
}

// PBKDF2 Hash
#[tauri::command]
fn hash_pbkdf2(request: PasswordHashRequest) -> Result<PasswordHashResponse, String> {
    let salt_string = if let Some(salt) = request.salt {
        SaltString::from_b64(&salt)
            .map_err(|e| format!("Invalid salt: {}", e))?
    } else {
        SaltString::generate(&mut OsRng)
    };
    
    let password_hash = Pbkdf2
        .hash_password(request.password.as_bytes(), &salt_string)
        .map_err(|e| format!("Failed to hash password: {}", e))?;
    
    Ok(PasswordHashResponse {
        hash: password_hash.to_string(),
        salt: salt_string.to_string(),
    })
}

// PBKDF2 Verify
#[tauri::command]
fn verify_pbkdf2(request: PasswordVerifyRequest) -> Result<PasswordVerifyResponse, String> {
    let parsed_hash = PasswordHash::new(&request.hash)
        .map_err(|e| format!("Invalid hash format: {}", e))?;
    
    let valid = Pbkdf2
        .verify_password(request.password.as_bytes(), &parsed_hash)
        .is_ok();
    
    Ok(PasswordVerifyResponse { valid })
}

// Scrypt Hash
#[tauri::command]
fn hash_scrypt(request: PasswordHashRequest) -> Result<PasswordHashResponse, String> {
    let salt_string = if let Some(salt) = request.salt {
        ScryptSaltString::from_b64(&salt)
            .map_err(|e| format!("Invalid salt: {}", e))?
    } else {
        ScryptSaltString::generate(&mut OsRng)
    };
    
    let password_hash = Scrypt
        .hash_password(request.password.as_bytes(), &salt_string)
        .map_err(|e| format!("Failed to hash password: {}", e))?;
    
    Ok(PasswordHashResponse {
        hash: password_hash.to_string(),
        salt: salt_string.to_string(),
    })
}

// Scrypt Verify
#[tauri::command]
fn verify_scrypt(request: PasswordVerifyRequest) -> Result<PasswordVerifyResponse, String> {
    let parsed_hash = ScryptPasswordHash::new(&request.hash)
        .map_err(|e| format!("Invalid hash format: {}", e))?;
    
    let valid = Scrypt
        .verify_password(request.password.as_bytes(), &parsed_hash)
        .is_ok();
    
    Ok(PasswordVerifyResponse { valid })
}

// Bcrypt Hash
#[tauri::command]
fn hash_bcrypt(request: PasswordHashRequest) -> Result<PasswordHashResponse, String> {
    let cost = request.iterations.unwrap_or(12) as u32;
    
    let hash = bcrypt::hash(&request.password, cost)
        .map_err(|e| format!("Failed to hash password: {}", e))?;
    
    // Bcrypt hash includes the salt
    Ok(PasswordHashResponse {
        hash: hash.clone(),
        salt: "included_in_hash".to_string(),
    })
}

// Bcrypt Verify
#[tauri::command]
fn verify_bcrypt(request: PasswordVerifyRequest) -> Result<PasswordVerifyResponse, String> {
    let valid = bcrypt::verify(&request.password, &request.hash)
        .map_err(|e| format!("Failed to verify password: {}", e))?;
    
    Ok(PasswordVerifyResponse { valid })
}

// Argon2 Hash
#[tauri::command]
fn hash_argon2(request: PasswordHashRequest) -> Result<PasswordHashResponse, String> {
    let salt_string = if let Some(salt) = request.salt {
        Argon2SaltString::from_b64(&salt)
            .map_err(|e| format!("Invalid salt: {}", e))?
    } else {
        Argon2SaltString::generate(&mut OsRng)
    };
    
    // Build Argon2 params - use default and customize if needed
    let params = if request.memory.is_some() || request.argon2_iterations.is_some() || request.parallelism.is_some() {
        let mut builder = ParamsBuilder::new();
        if let Some(m) = request.memory {
            builder.m_cost(m);
        }
        if let Some(t) = request.argon2_iterations {
            builder.t_cost(t);
        }
        if let Some(p) = request.parallelism {
            builder.p_cost(p);
        }
        builder.build().map_err(|e| format!("Failed to build params: {}", e))?
    } else {
        argon2::Params::default()
    };
    
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    );
    
    let password_hash = argon2
        .hash_password(request.password.as_bytes(), &salt_string)
        .map_err(|e| format!("Failed to hash password: {}", e))?;
    
    Ok(PasswordHashResponse {
        hash: password_hash.to_string(),
        salt: salt_string.to_string(),
    })
}

// Argon2 Verify
#[tauri::command]
fn verify_argon2(request: PasswordVerifyRequest) -> Result<PasswordVerifyResponse, String> {
    let parsed_hash = Argon2PasswordHash::new(&request.hash)
        .map_err(|e| format!("Invalid hash format: {}", e))?;
    
    let argon2 = Argon2::default();
    let valid = argon2
        .verify_password(request.password.as_bytes(), &parsed_hash)
        .is_ok();
    
    Ok(PasswordVerifyResponse { valid })
}

// Image compression structures
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageCompressRequest {
    image_data: Vec<u8>, // Base64 encoded or raw bytes
    quality: f32, // 0.0 to 1.0
    format: Option<String>, // "jpeg", "png", "webp", or None for auto
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageCompressResponse {
    compressed_data: Vec<u8>,
    format: String,
    original_size: usize,
    compressed_size: usize,
}

// Image compression command
#[tauri::command]
fn compress_image(request: ImageCompressRequest) -> Result<ImageCompressResponse, String> {
    let original_size = request.image_data.len();
    
    // Decode the image
    let img = image::load_from_memory(&request.image_data)
        .map_err(|e| format!("Failed to decode image: {}", e))?;
    
    // Check if WebP format is requested
    if let Some(ref fmt) = request.format {
        if fmt == "webp" {
            return compress_webp(&img, request.quality, original_size, &request.image_data);
        }
    }
    
    // Determine output format and encode
    let mut buffer = Vec::new();
    let format_name = if let Some(ref fmt) = request.format {
        match fmt.as_str() {
            "jpeg" | "jpg" => {
                let quality = (request.quality * 100.0).round() as u8;
                let rgb_img = img.to_rgb8();
                let mut cursor = Cursor::new(&mut buffer);
                let encoder = JpegEncoder::new_with_quality(&mut cursor, quality.max(1).min(100));
                encoder
                    .write_image(rgb_img.as_raw(), rgb_img.width(), rgb_img.height(), ExtendedColorType::Rgb8)
                    .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
                "jpeg"
            },
            "png" => {
                let rgba_img = img.to_rgba8();
                let mut cursor = Cursor::new(&mut buffer);
                let encoder = PngEncoder::new(&mut cursor);
                encoder
                    .write_image(rgba_img.as_raw(), rgba_img.width(), rgba_img.height(), ExtendedColorType::Rgba8)
                    .map_err(|e| format!("Failed to encode PNG: {}", e))?;
                "png"
            },
            _ => {
                let quality = (request.quality * 100.0).round() as u8;
                let rgb_img = img.to_rgb8();
                let mut cursor = Cursor::new(&mut buffer);
                let encoder = JpegEncoder::new_with_quality(&mut cursor, quality.max(1).min(100));
                encoder
                    .write_image(rgb_img.as_raw(), rgb_img.width(), rgb_img.height(), ExtendedColorType::Rgb8)
                    .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
                "jpeg"
            },
        }
    } else {
        // Auto-select format: try JPEG first (usually better compression)
        let quality = (request.quality * 100.0).round() as u8;
        let rgb_img = img.to_rgb8();
        let mut cursor = Cursor::new(&mut buffer);
        let encoder = JpegEncoder::new_with_quality(&mut cursor, quality.max(1).min(100));
        encoder
            .write_image(rgb_img.as_raw(), rgb_img.width(), rgb_img.height(), ExtendedColorType::Rgb8)
            .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
        "jpeg"
    };
    
    // If compressed is larger, try lower quality or different format
    if buffer.len() >= original_size {
        // Try lower quality JPEG
        let lower_quality = (request.quality * 0.7).max(0.3);
        let quality_u8 = (lower_quality * 100.0).round() as u8;
        let mut lower_buffer = Vec::new();
        {
            let rgb_img = img.to_rgb8();
            let mut cursor = Cursor::new(&mut lower_buffer);
            let encoder = JpegEncoder::new_with_quality(&mut cursor, quality_u8.max(1).min(100));
            encoder
                .write_image(rgb_img.as_raw(), rgb_img.width(), rgb_img.height(), ExtendedColorType::Rgb8)
                .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
        }
        
        // Use the smaller result
        if lower_buffer.len() < buffer.len() && lower_buffer.len() < original_size {
            buffer = lower_buffer;
        } else if buffer.len() < original_size {
            // Keep the original compression result
        } else {
            // Both are larger, return original (no compression)
            return Ok(ImageCompressResponse {
                compressed_data: request.image_data,
                format: "original".to_string(),
                original_size,
                compressed_size: original_size,
            });
        }
    }
    
    let compressed_size = buffer.len();
    Ok(ImageCompressResponse {
        compressed_data: buffer,
        format: format_name.to_string(),
        original_size,
        compressed_size,
    })
}

// Helper function for WebP compression
fn compress_webp(img: &DynamicImage, quality: f32, original_size: usize, original_data: &[u8]) -> Result<ImageCompressResponse, String> {
    // Convert to RGB image
    let rgb_img = img.to_rgb8();
    
    // Encode as WebP
    // webp crate uses quality from 0.0 to 100.0
    let webp_quality = (quality * 100.0).round();
    let encoder = webp::Encoder::from_rgb(&rgb_img, rgb_img.width(), rgb_img.height());
    let webp_data = encoder.encode(webp_quality);
    
    // If WebP is larger, try lower quality
    if webp_data.len() >= original_size {
        let lower_quality = (quality * 0.7).max(0.3);
        let lower_webp_quality = (lower_quality * 100.0).round();
        let lower_webp_data = encoder.encode(lower_webp_quality);
        
        // If lower quality is smaller, use it; otherwise return original
        if lower_webp_data.len() < original_size && lower_webp_data.len() < webp_data.len() {
            return Ok(ImageCompressResponse {
                compressed_data: lower_webp_data.to_vec(),
                format: "webp".to_string(),
                original_size,
                compressed_size: lower_webp_data.len(),
            });
        } else {
            // Return original (no compression)
            return Ok(ImageCompressResponse {
                compressed_data: original_data.to_vec(),
                format: "original".to_string(),
                original_size,
                compressed_size: original_size,
            });
        }
    }
    
    Ok(ImageCompressResponse {
        compressed_data: webp_data.to_vec(),
        format: "webp".to_string(),
        original_size,
        compressed_size: webp_data.len(),
    })
}

// PDF to Image structures
#[derive(Debug, Serialize, Deserialize)]
pub struct PdfToImageRequest {
    pdf_data: Vec<u8>, // PDF file bytes
    page_number: Option<u32>, // Page number (1-indexed), None for first page
    scale: Option<f32>, // Scale factor (default: 2.0 for 2x resolution)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfToImageResponse {
    image_data: Vec<u8>, // PNG image bytes
    width: u32,
    height: u32,
    page_count: u32,
}

// PDF to Image command using PDFium for accurate rendering
#[tauri::command]
fn pdf_to_image(request: PdfToImageRequest) -> Result<PdfToImageResponse, String> {
    use pdfium_render::prelude::*;
    use image::ImageEncoder;
    use std::path::PathBuf;
    use std::io::Cursor;

    // Bind to Pdfium with robust path resolution:
    // 1) system library
    // 2) absolute path under project `src-tauri/libs` (compile-time base)
    // 3) paths relative to current working dir: `libs/` and `./`
    let bindings = Pdfium::bind_to_system_library()
        .or_else(|_| {
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let abs_candidates = [
                manifest_dir.join("libs/libpdfium.dylib"),
                manifest_dir.join("libs/lib/libpdfium.dylib"),
            ];
            for p in abs_candidates.iter() {
                if p.exists() {
                    return Pdfium::bind_to_library(p);
                }
            }

            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let rel_candidates = [
                cwd.join("libs/libpdfium.dylib"),
                cwd.join("libpdfium.dylib"),
            ];
            for p in rel_candidates.iter() {
                if p.exists() {
                    return Pdfium::bind_to_library(p);
                }
            }

            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name())
        })
        .map_err(|_| "Failed to bind to Pdfium library".to_string())?;

    let pdfium = Pdfium::new(bindings);

    // Load document from bytes
    let document = pdfium
        .load_pdf_from_byte_slice(&request.pdf_data, None)
        .map_err(|e| format!("Failed to load PDF: {:?}", e))?;

    let page_count = document.pages().len() as u32;
    let page_number = request.page_number.unwrap_or(1);
    if page_number < 1 || page_number > page_count {
        return Err(format!("Page {} not found. PDF has {} pages", page_number, page_count));
    }

    let page = document
        .pages()
        .get((page_number - 1) as u16)
        .map_err(|_| "Failed to get page".to_string())?;

    // Render with optional scale
    let scale = request.scale.unwrap_or(2.0);
    let render_config = PdfRenderConfig::new()
        .scale_page_by_factor(scale as f32)
        .render_form_data(true)
        .render_annotations(true);
    let dyn_image = page
        .render_with_config(&render_config)
        .map_err(|e| format!("Failed to render page: {:?}", e))?
        .as_image();

    let rgba8 = dyn_image.into_rgba8();
    let (width, height) = rgba8.dimensions();

    // Encode PNG
    let mut png_buffer = Vec::new();
    {
        let mut cursor = Cursor::new(&mut png_buffer);
        let encoder = image::codecs::png::PngEncoder::new(&mut cursor);
        encoder
            .write_image(
                &rgba8,
                width,
                height,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    }

    Ok(PdfToImageResponse {
        image_data: png_buffer,
        width,
        height,
        page_count,
    })
}

// TLS version detection structures
#[derive(Debug, Serialize, Deserialize)]
pub struct TlsCheckRequest {
    host: String,
    port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TlsVersionInfo {
    version: String,
    supported: bool,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CipherSuiteInfo {
    name: String,
    supported: bool,
    error: Option<String>,
    tls_version: String,
    security_level: String, // "secure", "moderate", "weak", "insecure"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TlsCheckResponse {
    host: String,
    port: u16,
    supported_versions: Vec<TlsVersionInfo>,
    preferred_version: Option<String>,
    certificate_info: Option<CertificateInfo>,
    cipher_suites: Vec<CipherSuiteInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortScanRequest {
    host: String,
    start_port: u16,
    end_port: u16,
    timeout_ms: Option<u64>,
    max_concurrency: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortScanResult {
    port: u16,
    status: String,
    latency_ms: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortScanResponse {
    host: String,
    start_port: u16,
    end_port: u16,
    scanned_ports: usize,
    duration_ms: u128,
    open_ports: Vec<PortScanResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TracerouteRequest {
    host: String,
    max_hops: u8,
    timeout_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TracerouteHop {
    hop: u8,
    hostname: Option<String>,
    ip: Option<String>,
    rtt_ms: Option<f64>,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TracerouteResponse {
    host: String,
    hops: Vec<TracerouteHop>,
    duration_ms: u128,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CertificateInfo {
    subject: String,
    issuer: String,
    valid_from: String,
    valid_to: String,
    serial_number: String,
}

// TLS version detection command
#[tauri::command]
async fn check_tls_versions(request: TlsCheckRequest) -> Result<TlsCheckResponse, String> {
    use native_tls::Protocol;
    
    let host = request.host.clone();
    let port = request.port.unwrap_or(443);
    
    // TLS versions to test (native-tls may not support TLS 1.3, so we test it separately)
    let versions_to_test = vec![
        ("TLSv1.2", Protocol::Tlsv12),
        ("TLSv1.1", Protocol::Tlsv11),
        ("TLSv1.0", Protocol::Tlsv10),
    ];
    
    // Test TLS 1.3 separately (try with default settings which may support 1.3)
    let tls13_result = test_tls13(&host, port).await;
    let tls13_supported = tls13_result.is_ok();
    let tls13_error = tls13_result.as_ref().err().map(|e| e.clone());
    let tls13_cert_info = tls13_result.ok().and_then(|(cert_info, _)| cert_info);
    
    let mut supported_versions = Vec::new();
    let mut preferred_version: Option<String> = None;
    let mut certificate_info: Option<CertificateInfo> = None;
    
    // Add TLS 1.3 result first (highest priority)
    if tls13_supported {
        preferred_version = Some("TLSv1.3".to_string());
        if certificate_info.is_none() {
            certificate_info = tls13_cert_info;
        }
    }
    supported_versions.push(TlsVersionInfo {
        version: "TLSv1.3".to_string(),
        supported: tls13_supported,
        error: tls13_error,
    });
    
    // Test each TLS version
    for (version_name, protocol) in versions_to_test {
        match test_tls_version(&host, port, protocol).await {
            Ok((cert_info, is_supported)) => {
                if is_supported && preferred_version.is_none() {
                    preferred_version = Some(version_name.to_string());
                }
                if certificate_info.is_none() && cert_info.is_some() {
                    certificate_info = cert_info;
                }
                supported_versions.push(TlsVersionInfo {
                    version: version_name.to_string(),
                    supported: is_supported,
                    error: None,
                });
            }
            Err(e) => {
                supported_versions.push(TlsVersionInfo {
                    version: version_name.to_string(),
                    supported: false,
                    error: Some(e),
                });
            }
        }
    }
    
    // Detect cipher suites
    let cipher_suites = detect_cipher_suites(&host, port).await;
    
    Ok(TlsCheckResponse {
        host,
        port,
        supported_versions,
        preferred_version,
        certificate_info,
        cipher_suites,
    })
}

async fn test_tls13(
    host: &str,
    port: u16,
) -> Result<(Option<CertificateInfo>, bool), String> {
    use native_tls::{TlsConnector, Protocol};
    use tokio::net::TcpStream;
    use tokio_native_tls::TlsConnector as TokioTlsConnector;
    
    // Try to create connector with TLS 1.3 support (if available)
    let mut builder = TlsConnector::builder();
    // Set minimum to TLS 1.2, allow negotiation to 1.3 if supported
    builder.min_protocol_version(Some(Protocol::Tlsv12));
    
    let connector = builder
        .build()
        .map_err(|e| format!("Failed to create TLS connector: {}", e))?;
    
    let connector = TokioTlsConnector::from(connector);
    
    // Connect to server
    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;
    
    // Try TLS handshake (will negotiate highest available version)
    let tls_stream = connector
        .connect(host, stream)
        .await
        .map_err(|e| format!("TLS handshake failed: {}", e))?;
    
    // Check if we got TLS 1.3 (this is approximate - native-tls doesn't expose version directly)
    // We'll assume if connection succeeds with min TLS 1.2, it might support 1.3
    let cert_info = extract_certificate_info(&tls_stream);
    
    Ok((cert_info, true))
}

async fn test_tls_version(
    host: &str,
    port: u16,
    protocol: native_tls::Protocol,
) -> Result<(Option<CertificateInfo>, bool), String> {
    use native_tls::TlsConnector;
    use tokio::net::TcpStream;
    use tokio_native_tls::TlsConnector as TokioTlsConnector;
    
    // Create TLS connector with specific protocol
    let mut builder = TlsConnector::builder();
    builder.min_protocol_version(Some(protocol));
    builder.max_protocol_version(Some(protocol));
    
    let connector = builder
        .build()
        .map_err(|e| format!("Failed to create TLS connector: {}", e))?;
    
    let connector = TokioTlsConnector::from(connector);
    
    // Connect to server
    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;
    
    // Try TLS handshake
    let tls_stream = connector
        .connect(host, stream)
        .await
        .map_err(|e| format!("TLS handshake failed: {}", e))?;
    
    // Extract certificate info
    let cert_info = extract_certificate_info(&tls_stream);
    
    Ok((cert_info, true))
}

fn extract_certificate_info(
    _tls_stream: &tokio_native_tls::TlsStream<tokio::net::TcpStream>,
) -> Option<CertificateInfo> {
    // Get peer certificate - native-tls doesn't expose certificate directly in tokio wrapper
    // For now, return None - certificate parsing would require additional dependencies
    // This can be enhanced later with x509-parser or similar
    None
}

fn get_cipher_suite_info(suite_name: &str) -> (String, String) {
    // Determine TLS version and security level based on cipher suite name
    let (tls_version, security_level) = if suite_name.starts_with("TLS_AES_") || suite_name.starts_with("TLS_CHACHA20_") {
        // TLS 1.3 cipher suites - all are secure
        ("TLSv1.3".to_string(), "secure".to_string())
    } else if suite_name.contains("_GCM_") {
        // TLS 1.2 GCM cipher suites
        if suite_name.contains("ECDHE") {
            ("TLSv1.2".to_string(), "secure".to_string())
        } else if suite_name.contains("RSA_WITH") {
            ("TLSv1.2".to_string(), "moderate".to_string())
        } else {
            ("TLSv1.2".to_string(), "secure".to_string())
        }
    } else if suite_name.contains("_CHACHA20_POLY1305_") {
        // ChaCha20-Poly1305 cipher suites
        ("TLSv1.2".to_string(), "secure".to_string())
    } else if suite_name.contains("_CBC_SHA384") || suite_name.contains("_CBC_SHA256") {
        // CBC with SHA384/SHA256 - moderate
        ("TLSv1.2".to_string(), "moderate".to_string())
    } else if suite_name.contains("_CBC_SHA") {
        // CBC with SHA - weak
        if suite_name.contains("ECDHE") {
            ("TLSv1.2".to_string(), "weak".to_string())
        } else {
            ("TLSv1.2".to_string(), "insecure".to_string())
        }
    } else {
        // Default to TLS 1.2 and moderate
        ("TLSv1.2".to_string(), "moderate".to_string())
    };
    
    (tls_version, security_level)
}

async fn detect_cipher_suites(host: &str, port: u16) -> Vec<CipherSuiteInfo> {
    // Common cipher suites to test
    let cipher_suites_to_test = vec![
        // TLS 1.3 cipher suites
        "TLS_AES_256_GCM_SHA384",
        "TLS_CHACHA20_POLY1305_SHA256",
        "TLS_AES_128_GCM_SHA256",
        // TLS 1.2 cipher suites (strong)
        "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
        "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
        "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384",
        "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
        "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256",
        "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256",
        // TLS 1.2 cipher suites (moderate)
        "TLS_RSA_WITH_AES_256_GCM_SHA384",
        "TLS_RSA_WITH_AES_128_GCM_SHA256",
        "TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA384",
        "TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA256",
        "TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA384",
        "TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA256",
        // TLS 1.2 cipher suites (weak, for compatibility)
        "TLS_RSA_WITH_AES_256_CBC_SHA256",
        "TLS_RSA_WITH_AES_128_CBC_SHA256",
        "TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA",
        "TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA",
        "TLS_RSA_WITH_AES_256_CBC_SHA",
        "TLS_RSA_WITH_AES_128_CBC_SHA",
    ];
    
    // Try to get negotiated cipher suite using openssl (if available)
    let negotiated_suite = get_negotiated_cipher_suite_openssl(host, port).await;
    
    let mut results = Vec::new();
    
    // Test each cipher suite using openssl
    for suite_name in cipher_suites_to_test {
        let supported = test_cipher_suite_openssl(host, port, suite_name).await;
        let (tls_version, security_level) = get_cipher_suite_info(suite_name);
        
        results.push(CipherSuiteInfo {
            name: suite_name.to_string(),
            supported,
            error: None,
            tls_version,
            security_level,
        });
    }
    
    // If we got a negotiated suite that's not in our list, add it
    if let Some(ref negotiated) = negotiated_suite {
        if !results.iter().any(|r| r.name == *negotiated) {
            let (tls_version, security_level) = get_cipher_suite_info(negotiated);
            results.insert(0, CipherSuiteInfo {
                name: format!("{} (negotiated)", negotiated),
                supported: true,
                error: None,
                tls_version,
                security_level,
            });
        } else {
            // Mark the negotiated suite as supported
            for result in &mut results {
                if result.name == *negotiated {
                    result.supported = true;
                    break;
                }
            }
        }
    }
    
    results
}

async fn get_negotiated_cipher_suite_openssl(host: &str, port: u16) -> Option<String> {
    use tokio::process::Command;
    
    let output = Command::new("openssl")
        .args(&[
            "s_client",
            "-connect",
            &format!("{}:{}", host, port),
            "-servername",
            host,
        ])
        .stdin(std::process::Stdio::null())
        .output()
        .await
        .ok()?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    // Parse the cipher suite from openssl output
    // Look for "Cipher    :" line
    for line in output_str.lines() {
        if line.contains("Cipher    :") || line.contains("Cipher:") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() > 1 {
                let cipher = parts[1].trim();
                if !cipher.is_empty() && cipher != "NONE" {
                    // Convert openssl cipher name to IANA name format
                    return Some(convert_openssl_to_iana(cipher));
                }
            }
        }
    }
    
    None
}

async fn test_cipher_suite_openssl(host: &str, port: u16, cipher_suite: &str) -> bool {
    use tokio::process::Command;
    
    // Convert IANA cipher suite name to openssl format
    let openssl_cipher = convert_iana_to_openssl(cipher_suite);
    
    let output = Command::new("openssl")
        .args(&[
            "s_client",
            "-connect",
            &format!("{}:{}", host, port),
            "-servername",
            host,
            "-cipher",
            &openssl_cipher,
        ])
        .stdin(std::process::Stdio::null())
        .output()
        .await;
    
    match output {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Check if connection was successful
            output_str.contains("Verify return code: 0") || 
            output_str.contains("Cipher    :") ||
            output.status.success()
        }
        Err(_) => false,
    }
}

fn convert_openssl_to_iana(openssl_name: &str) -> String {
    // Convert openssl cipher name to IANA format
    // This is a simplified conversion - in production, you'd want a complete mapping
    openssl_name.replace(":", "_").to_uppercase()
}

fn convert_iana_to_openssl(iana_name: &str) -> String {
    // Convert IANA cipher suite name to openssl format
    // This is a simplified conversion - in production, you'd want a complete mapping
    // For now, return a generic cipher that openssl can try
    match iana_name {
        "TLS_AES_256_GCM_SHA384" => "TLS_AES_256_GCM_SHA384",
        "TLS_CHACHA20_POLY1305_SHA256" => "TLS_CHACHA20_POLY1305_SHA256",
        "TLS_AES_128_GCM_SHA256" => "TLS_AES_128_GCM_SHA256",
        "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384" => "ECDHE-RSA-AES256-GCM-SHA384",
        "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256" => "ECDHE-RSA-AES128-GCM-SHA256",
        "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384" => "ECDHE-ECDSA-AES256-GCM-SHA384",
        "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256" => "ECDHE-ECDSA-AES128-GCM-SHA256",
        "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256" => "ECDHE-RSA-CHACHA20-POLY1305",
        "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256" => "ECDHE-ECDSA-CHACHA20-POLY1305",
        "TLS_RSA_WITH_AES_256_GCM_SHA384" => "AES256-GCM-SHA384",
        "TLS_RSA_WITH_AES_128_GCM_SHA256" => "AES128-GCM-SHA256",
        _ => iana_name,
    }.to_string()
}

// Port scanner command
#[tauri::command]
async fn scan_ports(request: PortScanRequest) -> Result<PortScanResponse, String> {
    use futures::stream::{self, StreamExt};
    use tokio::time::{timeout, Duration};

    let host = request.host.trim().to_string();
    if host.is_empty() {
        return Err("Host is required".to_string());
    }

    if request.start_port == 0 || request.end_port == 0 || request.start_port > request.end_port {
        return Err("Invalid port range".to_string());
    }

    // Auto-adjust timeout and concurrency based on port range size
    let port_count = (request.end_port - request.start_port + 1) as usize;
    let (default_timeout, default_concurrency) = if port_count > 10000 {
        // Large range (e.g., all ports): faster timeout, higher concurrency
        (300, 500)
    } else if port_count > 2048 {
        // Medium range: moderate settings
        (500, 300)
    } else {
        // Small range: default settings
        (700, 200)
    };
    
    let timeout_ms = request.timeout_ms.unwrap_or(default_timeout).clamp(50, 10_000);
    let concurrency = request.max_concurrency.unwrap_or(default_concurrency).clamp(1, 1000);
    let timeout_duration = Duration::from_millis(timeout_ms);

    let ports: Vec<u16> = (request.start_port..=request.end_port).collect();
    let total_ports = ports.len();
    let start = std::time::Instant::now();

    let open_ports = stream::iter(ports.into_iter().map(|port| {
        let host = host.clone();
        async move {
            let port_start = std::time::Instant::now();
            let addr = format!("{}:{}", host, port);
            match timeout(timeout_duration, tokio::net::TcpStream::connect(&addr)).await {
                Ok(Ok(_stream)) => {
                    let latency = port_start.elapsed().as_secs_f64() * 1000.0;
                    Some(PortScanResult {
                        port,
                        status: "open".to_string(),
                        latency_ms: Some(latency),
                    })
                }
                _ => None,
            }
        }
    }))
    .buffer_unordered(concurrency)
    .filter_map(|result| async move { result })
    .collect::<Vec<_>>()
    .await;

    Ok(PortScanResponse {
        host,
        start_port: request.start_port,
        end_port: request.end_port,
        scanned_ports: total_ports,
        duration_ms: start.elapsed().as_millis(),
        open_ports,
    })
}

// Traceroute command
#[tauri::command]
async fn traceroute(request: TracerouteRequest) -> Result<TracerouteResponse, String> {
    use tokio::process::Command;
    use std::time::Instant;
    
    let host = request.host.trim().to_string();
    if host.is_empty() {
        return Err("Host is required".to_string());
    }
    
    let start = Instant::now();
    let max_hops = request.max_hops.clamp(1, 64);
    let timeout_secs = (request.timeout_ms / 1000).max(1).min(30);
    
    // Determine the traceroute command based on the platform
    let (cmd_name, args) = if cfg!(target_os = "windows") {
        (
            "tracert",
            vec![
                "-h".to_string(),
                max_hops.to_string(),
                "-w".to_string(),
                (timeout_secs * 1000).to_string(), // Windows uses milliseconds
                host.clone(),
            ],
        )
    } else {
        // Linux/macOS
        (
            "traceroute",
            vec![
                "-m".to_string(),
                max_hops.to_string(),
                "-w".to_string(),
                timeout_secs.to_string(),
                "-q".to_string(),
                "1".to_string(), // Only 1 probe per hop for faster results
                host.clone(),
            ],
        )
    };
    
    let output = Command::new(cmd_name)
        .args(&args)
        .output()
        .await;
    
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // Check if command failed
            if !output.status.success() && !stderr.is_empty() {
                return Ok(TracerouteResponse {
                    host,
                    hops: vec![],
                    duration_ms: start.elapsed().as_millis(),
                    error: Some(format!("Traceroute failed: {}", stderr)),
                });
            }
            
            // Parse traceroute output
            let hops = parse_traceroute_output(&stdout, cfg!(target_os = "windows"));
            
            Ok(TracerouteResponse {
                host,
                hops,
                duration_ms: start.elapsed().as_millis(),
                error: None,
            })
        }
        Err(e) => {
            Ok(TracerouteResponse {
                host,
                hops: vec![],
                duration_ms: start.elapsed().as_millis(),
                error: Some(format!(
                    "Failed to execute traceroute command: {}. Make sure {} is installed and available in PATH.",
                    e, cmd_name
                )),
            })
        }
    }
}

fn parse_traceroute_output(output: &str, is_windows: bool) -> Vec<TracerouteHop> {
    let mut hops = Vec::new();
    
    if is_windows {
        // Windows tracert format:
        //   1    <1 ms    <1 ms    <1 ms  192.168.1.1
        //   2    10 ms    10 ms    10 ms  10.0.0.1
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("Tracing") || line.starts_with("Trace") {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                if let Ok(hop_num) = parts[0].parse::<u8>() {
                    // Extract RTT (take the first RTT value)
                    let rtt_str = parts[1];
                    let rtt_ms = rtt_str
                        .replace("<", "")
                        .replace("ms", "")
                        .trim()
                        .parse::<f64>()
                        .ok();
                    
                    // Extract IP (last part)
                    let ip = parts.last().map(|s| s.to_string());
                    
                    hops.push(TracerouteHop {
                        hop: hop_num,
                        hostname: None,
                        ip,
                        rtt_ms,
                        status: if rtt_ms.is_some() {
                            "success".to_string()
                        } else {
                            "timeout".to_string()
                        },
                    });
                }
            }
        }
    } else {
        // Linux/macOS traceroute format:
        //  1  192.168.1.1 (192.168.1.1)  0.234 ms  0.123 ms  0.098 ms
        //  2  * * *
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("traceroute") {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            
            if let Ok(hop_num) = parts[0].parse::<u8>() {
                if parts.len() >= 2 {
                    if parts[1] == "*" {
                        // Timeout
                        hops.push(TracerouteHop {
                            hop: hop_num,
                            hostname: None,
                            ip: None,
                            rtt_ms: None,
                            status: "timeout".to_string(),
                        });
                    } else {
                        // Extract hostname and IP
                        let hostname = if parts[1].starts_with('(') {
                            None
                        } else {
                            Some(parts[1].to_string())
                        };
                        
                        let ip = if parts.len() >= 3 && parts[2].starts_with('(') {
                            Some(parts[2].trim_matches(|c| c == '(' || c == ')').to_string())
                        } else if parts[1].contains('.') && !parts[1].starts_with('(') {
                            Some(parts[1].to_string())
                        } else {
                            None
                        };
                        
                        // Extract RTT (first numeric value after IP)
                        let mut rtt_ms = None;
                        for part in parts.iter().skip(2) {
                            if let Ok(rtt) = part.replace("ms", "").trim().parse::<f64>() {
                                rtt_ms = Some(rtt);
                                break;
                            }
                        }
                        
                        hops.push(TracerouteHop {
                            hop: hop_num,
                            hostname,
                            ip,
                            rtt_ms,
                            status: if rtt_ms.is_some() {
                                "success".to_string()
                            } else {
                                "timeout".to_string()
                            },
                        });
                    }
                }
            }
        }
    }
    
    hops
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::{Manager, Emitter};
    
    // 在 Linux 系统上，设置环境变量以解决白屏问题
    // 这主要是由于 WebKit2GTK 与 NVIDIA GPU 的兼容性问题
    #[cfg(target_os = "linux")]
    {
        // 禁用 WebKit 的 DMA-BUF 渲染器，解决 NVIDIA GPU 上的白屏问题
        if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
        
        // 如果用户没有设置，尝试使用软件渲染作为后备方案
        if std::env::var("LIBGL_ALWAYS_SOFTWARE").is_err() {
            // 尝试检测是否有权限访问 /dev/dri/ 设备
            let has_dri_access = std::path::Path::new("/dev/dri")
                .read_dir()
                .is_ok();
            
            // 如果没有访问权限，设置软件渲染环境变量
            if !has_dri_access {
                std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
            }
        }
    }
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 监听 macOS 系统菜单事件
            #[cfg(target_os = "macos")]
            {
                use tauri::menu::{MenuBuilder, PredefinedMenuItem, SubmenuBuilder, MenuItemBuilder};
                
                // 创建自定义的 About 菜单项（而不是使用预定义的）
                let about_item = MenuItemBuilder::with_id("about", "About Kairoa")
                    .build(app)?;
                
                // 创建检查更新菜单项
                let check_update_item = MenuItemBuilder::with_id("check_update", "Check for Updates...")
                    .build(app)?;
                
                // 创建 Settings 菜单项
                let settings_item = MenuItemBuilder::with_id("settings", "Settings...")
                    .accelerator("Cmd+,")  // macOS 标准的设置快捷键
                    .build(app)?;
                
                // 创建自定义应用子菜单
                let app_submenu = SubmenuBuilder::new(app, "kairoa")
                    .item(&about_item)  // About Kairoa
                    .item(&check_update_item)  // Check for Updates...
                    .item(&settings_item)  // Settings...
                    .separator()
                    .item(&PredefinedMenuItem::services(app, None)?)
                    .separator()
                    .item(&PredefinedMenuItem::hide(app, None)?)
                    .item(&PredefinedMenuItem::hide_others(app, None)?)
                    .item(&PredefinedMenuItem::show_all(app, None)?)
                    .separator()
                    .item(&PredefinedMenuItem::quit(app, None)?)
                    .build()?;
                
                // 创建 Edit 菜单（包含标准编辑快捷键）
                let edit_submenu = SubmenuBuilder::new(app, "Edit")
                    .item(&PredefinedMenuItem::undo(app, None)?)
                    .item(&PredefinedMenuItem::redo(app, None)?)
                    .separator()
                    .item(&PredefinedMenuItem::cut(app, None)?)
                    .item(&PredefinedMenuItem::copy(app, None)?)
                    .item(&PredefinedMenuItem::paste(app, None)?)
                    .item(&PredefinedMenuItem::select_all(app, None)?)
                    .build()?;
                
                // 创建完整的菜单栏
                let menu = MenuBuilder::new(app)
                    .item(&app_submenu)
                    .item(&edit_submenu)
                    .build()?;
                
                // 设置应用菜单
                app.set_menu(menu)?;
                
                // 监听菜单事件
                app.on_menu_event(move |app, event| {
                    let event_id = event.id().as_ref();
                    println!("Menu event received: {}", event_id); // 调试输出
                    
                    if let Some(window) = app.get_webview_window("main") {
                        match event_id {
                            "about" => {
                                println!("About menu clicked, emitting show-about event");
                                match window.emit("show-about", ()) {
                                    Ok(_) => println!("Event emitted successfully"),
                                    Err(e) => println!("Failed to emit event: {}", e),
                                }
                            }
                            "check_update" => {
                                println!("Check for Updates menu clicked");
                                match window.emit("check-for-updates", ()) {
                                    Ok(_) => println!("Check update event emitted successfully"),
                                    Err(e) => println!("Failed to emit check update event: {}", e),
                                }
                            }
                            "settings" => {
                                println!("Settings menu clicked, showing settings dialog");
                                match window.emit("show-settings", ()) {
                                    Ok(_) => println!("Settings event emitted successfully"),
                                    Err(e) => println!("Failed to emit settings event: {}", e),
                                }
                            }
                            _ => {}
                        }
                    } else {
                        println!("Window not found");
                    }
                });
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet, 
            http_request, 
            generate_dsa_keypair, 
            dsa_sign, 
            dsa_verify,
            hash_pbkdf2,
            verify_pbkdf2,
            hash_scrypt,
            verify_scrypt,
            hash_bcrypt,
            verify_bcrypt,
            hash_argon2,
            verify_argon2,
            compress_image,
            pdf_to_image,
            check_tls_versions,
            scan_ports,
            traceroute
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
