use sarge::{ArgParseError, ArgumentType};

use crate::cli::{InputArgs, OutputArgs};

impl ArgumentType for InputArgs {
    type Error = ArgParseError;

    fn from_value(val: Option<&str>) -> sarge::ArgResult<Self> {
        // 需要自动的识别用户输入的 参数是 path? content? 还是  stdin

        todo!()
    }
}

impl ArgumentType for OutputArgs {
    type Error = ArgParseError;

    fn from_value(val: Option<&str>) -> sarge::ArgResult<Self> {
        // 需要自动的识别用户输入的 参数是 path? 还是  stdout/stderr

        todo!()
    }
}
