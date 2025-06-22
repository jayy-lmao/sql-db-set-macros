//! Query builder modules for different types of database operations.
//!
//! This module contains implementations for:
//! - `many_query_builder`: For fetching multiple records with optional filtering
//! - `one_query_builder`: For fetching single records by key or unique fields
//! - `insert_query_builder`: For inserting new records
//! - `update_query_builder`: For updating existing records
//! - `delete_query_builder`: For deleting records
//! - `from_row`: For implementing `sqlx::FromRow` trait
//! - `dbset`: For the main DbSet implementation

pub mod dbset;
pub mod delete_query_builder;
pub mod from_row;
pub mod insert_query_builder;
pub mod many_query_builder;
pub mod one_query_builder;
pub mod update_query_builder;
pub mod query_builder_shared; // Add shared helpers module
