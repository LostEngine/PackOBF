use crate::options::ShaderCompression;
use crate::shader_minifier::minifier::Minifier;
use crate::LogLevel::Warning;
use crate::LogMessage;

#[derive(Clone, Debug)]
pub struct Shader {
    pub path: String,
    pub content: String,
}

impl Shader {
    pub fn new(path: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
        }
    }

    pub fn optimize(
        path: &String,
        content: String,
        options: &crate::options::Options,
        logger: &tokio::sync::mpsc::UnboundedSender<LogMessage>,
    ) -> String {
        if options.shader_compression == ShaderCompression::None {
            return content;
        }
        match Minifier::default().minify(
            &content,
            options.shader_compression == ShaderCompression::MinifyAndObfuscate,
        ) {
            Ok(minified_code) => {
                if !minified_code.is_empty() && minified_code != content {
                    return minified_code;
                }
                content
            }
            Err(e) => {
                let _ = logger.send(LogMessage {
                    level: Warning,
                    message: format!(
                        "Could not minify shader '{}'. Skipping optimization. Error: {}",
                        path, e
                    ),
                });
                content
            }
        }
    }
}
