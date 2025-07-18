use super::db::{AppError, DbAccess};

use crate::models::models::{NewUserBankAccount, UserBankAccount};
use crate::models::schema::user_bank_account::dsl::*;
use diesel::prelude::*;
use diesel::sql_types::Text;

diesel::define_sql_function! {
    fn lower(x: Text) -> Text;
}

pub trait UserBankImpl: DbAccess {
    fn create_user_bank(
        &self,
        bank_details: NewUserBankAccount,
    ) -> Result<UserBankAccount, AppError> {
        let mut conn = self.conn().map_err(AppError::DbConnectionError)?;

        diesel::insert_into(user_bank_account)
            .values(&bank_details)
            .get_result(&mut conn)
            .map_err(AppError::DieselError)
    }

    fn get_wallet_by_id(&self, bank_account_id: uuid::Uuid) -> Result<UserBankAccount, AppError> {
        let mut conn = self.conn().map_err(AppError::DbConnectionError)?;

        user_bank_account
            .find(bank_account_id)
            .first::<UserBankAccount>(&mut conn)
            .map_err(AppError::DieselError)
    }

    fn get_banks_by_user_id(
        &self,
        find_user: &str,
    ) -> Result<Vec<UserBankAccount>, AppError> {
        let mut conn = self.conn().map_err(AppError::DbConnectionError)?;

        user_bank_account
            .filter(user_id.eq(find_user))
            .get_results::<UserBankAccount>(&mut conn)
            .map_err(AppError::DieselError)
    }

    fn get_bank_by_account_number(
        &self,
        find_account_number: String,
    ) -> Result<UserBankAccount, AppError> {
        let mut conn = self.conn().map_err(AppError::DbConnectionError)?;

        user_bank_account
            .filter(account_number.eq(find_account_number))
            .first::<UserBankAccount>(&mut conn)
            .map_err(AppError::DieselError)
    }

    fn get_bank_by_account_number_and_user_id(
        &self,
        find_account_number: String,
        find_user: String,
    ) -> Result<UserBankAccount, AppError> {
        let mut conn = self.conn().map_err(AppError::DbConnectionError)?;

        user_bank_account
            .filter(account_number.eq(find_account_number))
            .filter(user_id.eq(find_user))
            .first::<UserBankAccount>(&mut conn)
            .map_err(AppError::DieselError)
    }
}
