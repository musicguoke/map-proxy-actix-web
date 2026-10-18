#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    TileNotFound,
    ServiceNotFound(String),
    ApiCallFound(String),
    PoolError(String),
}

// 实现 Display trait
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::DatabaseError(e) => write!(f, "数据库错误: {}", e),
            AppError::TileNotFound => write!(f, "瓦片未找到"),
            AppError::ServiceNotFound(s) => write!(f, "服务未找到: {}", s),
            AppError::PoolError(s) => write!(f, "连接池错误: {}", s),
            AppError::ApiCallFound(s) => write!(f, "第三方接口调用错误: {}", s),
        }
    }
}

// 实现 Error trait
impl std::error::Error for AppError {}

// 实现 From<sqlx::Error> 以便自动转换
impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError::DatabaseError(error)
    }
}
