use crate::parser::util::consume_till_empty_line;
use crate::parser::{Description, Error, OutputStream, ParsableFromStream};
use async_trait::async_trait;
use std::fmt::Display;
use std::path::PathBuf;
use tap::Pipe;

/// Storyboard compilation Step
#[derive(Debug)]
pub struct CompileStoryboard {
    pub description: Description,
    pub path: PathBuf,
}

#[async_trait]
impl ParsableFromStream for CompileStoryboard {
    async fn parse_from_stream(line: String, stream: &mut OutputStream) -> Result<Self, Error> {
        let mut chunks = line.split_whitespace();
        let path = chunks
            .next()
            .map(PathBuf::from)
            .ok_or_else(|| Error::EOF("CompileStoryboard".into(), "path".into()))?;

        let description = Description::from_line(line)?;

        consume_till_empty_line(stream).await;

        Self { description, path }.pipe(Ok)
    }
}

impl Display for CompileStoryboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} Compiling   {}",
            self.description,
            self.path.file_name().unwrap().to_string_lossy()
        )
    }
}

#[tokio::test]
#[cfg_attr(feature = "tracing", tracing_test::traced_test)]
async fn test() {
    use crate::parser::util::test::to_stream_test;

    let step = to_stream_test! {
        CompileStoryboard,
       r#"CompileStoryboard /path/to/a.storyboard (in target 'DemoTarget' from project 'DemoProject')
    cd /Users/tami5/repos/swift/wordle
    export XCODE_DEVELOPER_USR_PATH\=/Applications/Xcode.app/Contents/Developer/usr/bin/..
    /Applications/Xcode.app/Contents/Developer/usr/bin/ibtool ...

"# 
    };
    assert_eq!("DemoTarget", &step.description.target);
    assert_eq!("DemoProject", &step.description.project);
    assert_eq!(PathBuf::from("/path/to/a.storyboard"), step.path);
    assert_eq!(
        "[DemoProject.DemoTarget] Compiling `a.storyboard`",
        step.to_string()
    )
}