//! Constants used throughout the library.

/// SQL parameter placeholder prefix
pub const PARAM_PLACEHOLDER: &str = "$";

/// Default table name suffix when not specified
pub const DEFAULT_TABLE_SUFFIX: &str = "s";

/// Default DbSet name suffix
pub const DBSET_SUFFIX: &str = "DbSet";

/// Query builder type suffixes
pub const MANY_QUERY_BUILDER_SUFFIX: &str = "ManyQueryBuilder";
pub const ONE_QUERY_BUILDER_SUFFIX: &str = "OneQueryBuilder";
pub const INSERT_QUERY_BUILDER_SUFFIX: &str = "InsertBuilder";
pub const UPDATE_QUERY_BUILDER_SUFFIX: &str = "UpdateBuilder";
pub const DELETE_QUERY_BUILDER_SUFFIX: &str = "DeleteBuilder";

/// SQL keywords and operators
pub const SQL_SELECT: &str = "SELECT";
pub const SQL_FROM: &str = "FROM";
pub const SQL_WHERE: &str = "WHERE";
pub const SQL_INSERT: &str = "INSERT INTO";
pub const SQL_UPDATE: &str = "UPDATE";
pub const SQL_DELETE: &str = "DELETE FROM";
pub const SQL_SET: &str = "SET";
pub const SQL_VALUES: &str = "VALUES";
pub const SQL_RETURNING: &str = "RETURNING";
pub const SQL_AND: &str = "AND";
pub const SQL_OR: &str = "OR";
