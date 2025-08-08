use super::db::{AppError, DbAccess};
use crate::models::models::{NewToken, Token};
use crate::models::schema::user_jwt_tokens::dsl::*;
use diesel::prelude::*;
use diesel::sql_types::Text;

diesel::define_sql_function! {
    fn lower(x: Text) -> Text;
}

pub trait TokenImpl: DbAccess {
    fn create_token(&self, new_token: NewToken) -> Result<Token, AppError> {
        let mut conn = self.conn().map_err(AppError::DbConnectionError)?;

        diesel::insert_into(user_jwt_tokens)
            .values(&new_token)
            .get_result(&mut conn)
            .map_err(AppError::DieselError)
    }

    fn get_token_by_user_id(&self, find_user: String) -> Result<Token, AppError> {
        let mut conn = self.conn().map_err(AppError::DbConnectionError)?;

        user_jwt_tokens
            .filter(user_id.eq(find_user))
            .first::<Token>(&mut conn)
            .map_err(AppError::DieselError)
    }

    fn get_token_by_token_string(&self, find_token: String) -> Result<Token, AppError> {
        let mut conn = self.conn().map_err(AppError::DbConnectionError)?;

        user_jwt_tokens
            .filter(token.eq(find_token))
            .first::<Token>(&mut conn)
            .map_err(AppError::DieselError)
    }

    fn delete_token_by_id(&self, find_id: uuid::Uuid) -> Result<Token, AppError> {
        let mut conn = self.conn().map_err(AppError::DbConnectionError)?;

        diesel::delete(user_jwt_tokens.filter(token_id.eq(find_id)))
            .get_result::<Token>(&mut conn)
            .map_err(AppError::DieselError)
    }

    fn delete_tokens_by_user_id(&self, find_user: String) -> Result<Vec<Token>, AppError> {
        let mut conn = self.conn().map_err(AppError::DbConnectionError)?;

        diesel::delete(user_jwt_tokens.filter(user_id.eq(find_user)))
            .get_results(&mut conn)
            .map_err(AppError::DieselError)
    }
}
