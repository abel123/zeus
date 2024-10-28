use std::cell::RefCell;
use std::rc::Rc;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{rt, App, HttpServer};

use crate::api;
use crate::broker::mixed::Mixed;
use tws_rs::Error;

use crate::db::establish_connection;

pub fn serve() -> std::io::Result<()> {
    rt::System::new().block_on(
        HttpServer::new(|| {
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_header()
                .allow_any_method()
                .max_age(3600);
            App::new()
                .wrap(Logger::new("%a  %r %s %b  %T"))
                .wrap(cors)
                .data_factory(|| async { Ok::<_, Error>(Rc::new(RefCell::new(Mixed::new()))) })
                .data_factory(|| async {
                    let conn = establish_connection();

                    Ok::<_, Error>(RefCell::new(conn))
                })
                .service(api::history)
                .service(api::search_symbol)
                .service(api::resolve_symbol)
                .service(api::zen_element)
                .service(api::config)
                .service(api::option_price)
                .service(api::websocket)
        })
        .workers(1)
        .bind(("0.0.0.0", 8080))?
        .run(),
    )
}
