use super::super::{
    util::consume_empty_lines, Description, Error, OutputStream, ParsableFromStream,
};
use async_trait::async_trait;
use std::path::PathBuf;
use tap::Pipe;

#[derive(Debug)]
/// Clang compilation step
pub struct CompileC {
    pub compiler: String,
    pub description: Description,
    pub output_path: PathBuf,
    pub path: PathBuf,
    pub arch: String,
    pub lang: String,
}

#[async_trait]
impl ParsableFromStream for CompileC {
    async fn parse_from_stream(line: String, stream: &mut OutputStream) -> Result<Self, Error> {
        let mut chunks = line.split_whitespace();
        let output_path = chunks
            .next()
            .map(PathBuf::from)
            .ok_or_else(|| Error::EOF("CompileC".into(), "output_path".into()))?;

        let path = chunks
            .next()
            .map(PathBuf::from)
            .ok_or_else(|| Error::EOF("CompileC".into(), "path".into()))?;

        chunks.next();

        let arch = chunks
            .next()
            .map(ToString::to_string)
            .ok_or_else(|| Error::EOF("CompileC".into(), "path".into()))?;

        let lang = chunks
            .next()
            .map(ToString::to_string)
            .ok_or_else(|| Error::EOF("CompileC".into(), "path".into()))?;

        let compiler = chunks
            .next()
            .map(ToString::to_string)
            .ok_or_else(|| Error::EOF("CompileC".into(), "path".into()))?;

        #[cfg(feature = "tracing")]
        tracing::trace!("left {}", chunks.as_str());

        let description = Description::from_line(line)?;

        consume_empty_lines(stream).await;

        Self {
            compiler,
            description,
            output_path,
            path,
            arch,
            lang,
        }
        .pipe(Ok)
    }
}

#[tokio::test]
#[cfg_attr(feature = "tracing", tracing_test::traced_test)]
async fn test() {
    use crate::parser::util::test::to_stream_test;

    let step = to_stream_test! {
        CompileC,
       r#"CompileC path/to/output/bridge.o path/to/input/bridge.c normal arm64 c com.apple.compilers.llvm.clang.1_0.compiler (in target 'DemoTarget' from project 'DemoProject')
    cd $ROOT
    export LANG\=en_US.US-ASCII
    /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang ...

    "# 
    };
    assert_eq!("DemoTarget", &step.description.target);
    assert_eq!("DemoProject", &step.description.project);

    assert_eq!("arm64", &step.arch);
    assert_eq!("c", &step.lang);
    assert_eq!(PathBuf::from("path/to/input/bridge.c"), step.path);
    assert_eq!(PathBuf::from("path/to/output/bridge.o"), step.output_path);
}
