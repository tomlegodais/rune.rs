macro_rules! with_shutdown {
    ($writer: expr, $result: expr) => {{
        let result = $result;
        let _ = $writer.shutdown().await;
        result
    }};
}

pub(crate) use with_shutdown;
