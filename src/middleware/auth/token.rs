//! Auth Token
use super::{error, header};
use crate::{crypto::Address, share::Shared};
use actix_web::{dev::ServiceRequest, Error};
use redis::Commands;
use uuid::Uuid;

/// # No Token
///
/// Account doesn't have token when they login cdr.today for
/// the first time, as result, we'll return 401 and set a uuid
/// to the client.
///
/// # Has token
///
/// If have token in header, check the database to find if the
/// token is paired.
pub fn token(req: &ServiceRequest, address: &String) -> Result<(), Error> {
    if let Some(token) = req.headers().get(header::TOKEN) {
        let address = Address::from_str(&address).map_err(|_| error::AuthError::AddressInvalid)?;
        if !address
            .verify(&hex::decode(token).map_err(|_| error::AuthError::AddressInvalid)?)
            .map_err(|_| error::AuthError::AddressInvalid)?
        {
            Err(error::AuthError::TokenInvalid {
                uuid: Uuid::new_v4().to_string(),
            }
            .into())
        } else {
            Ok(())
        }
    } else {
        let uuid = Uuid::new_v4().to_string();
        if let Some(data) = req.app_data::<Shared>() {
            let _: () = data
                .redis
                .conn()
                .map_err(|_| {
                    actix_web::error::ErrorInternalServerError("Get redis connection failed")
                })?
                .set(address, &uuid)
                .map_err(|_| {
                    actix_web::error::ErrorInternalServerError("Set uuid into redis failed")
                })?;
        }

        Err(error::AuthError::TokenNotFound { uuid }.into())
    }
}
