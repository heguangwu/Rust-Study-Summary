
pub type CalcResult<T> = Result<T, CalcError>;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum CalcError {
    #[error("非法字符：{0}")]
    UnexpectedChar(char),
    #[error("无效运算符：{0}")]
    InvalidOperator(String)
}