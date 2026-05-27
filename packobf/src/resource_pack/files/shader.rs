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
        &mut self,
        options: &crate::options::Options,
        logger: &tokio::sync::mpsc::UnboundedSender<LogMessage>,
    ) {
        if options.shader_compression == ShaderCompression::None {
            return;
        }
        match Minifier::default().minify(
            &self.content,
            options.shader_compression == ShaderCompression::MinifyAndObfuscate,
        ) {
            Ok(minified_code) => {
                if !minified_code.is_empty() && minified_code != self.content {
                    self.content = minified_code;
                }
            }
            Err(e) => {
                let _ = logger.send(LogMessage {
                    level: Warning,
                    message: format!(
                        "Could not minify shader '{}'. Skipping optimization. Error: {}",
                        self.path, e
                    ),
                });
            }
        }
    }
}
