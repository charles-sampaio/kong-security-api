use actix_web::web;
use crate::handlers::{register, login, profile};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register);
    cfg.service(login);
    cfg.service(profile);
}
