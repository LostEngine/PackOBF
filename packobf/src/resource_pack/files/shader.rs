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
        &self,
        options: &crate::options::Options,
        logger: &tokio::sync::mpsc::UnboundedSender<LogMessage>,
    ) -> String {
        if options.shader_compression == ShaderCompression::None {
            return self.content.clone();
        }
        match Minifier::default().minify(
            &self.content,
            options.shader_compression == ShaderCompression::MinifyAndObfuscate,
        ) {
            Ok(minified_code) => {
                if !minified_code.is_empty() && minified_code != self.content {
                    return minified_code;
                }
                self.content.clone()
            }
            Err(e) => {
                let _ = logger.send(LogMessage {
                    level: Warning,
                    message: format!(
                        "Could not minify shader '{}'. Skipping optimization. Error: {}",
                        self.path, e
                    ),
                });
                self.content.clone()
            }
        }
    }
}
