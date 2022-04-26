use std::sync::Arc;
use std::sync::Mutex;

use rocket::form::Form;
use rocket::response::content::{Html, Json};
use urlencoding::decode;

use super::request::*;
use super::super::proxy::item::*;

fn decode_url_string(urlstr: &str) -> Option<String> {
    match decode(urlstr) {
        Ok(c) => Some(c.to_string()),
        Err(_) => None
    }
}

pub type LopxyManagerServerControllerArc = Arc<Mutex<dyn LopxyManagerServerController + Send>>;

pub trait LopxyManagerServerController {
    fn shutdown(&mut self);

    fn list_all_proxy_item(&mut self) -> &Vec<ProxyItem>;
    fn add_proxy_item(&mut self, resource_url: &str, proxy_resource_url: &str, content_type: &str) -> bool;
    fn remove_proxy_item(&mut self, resource_url: &str) -> bool;
    fn modify_proxy_item(&mut self, resource_url: &str, proxy_resource_url: &str, content_type: &str) -> bool;

    fn is_system_proxy_enabled(&mut self) -> bool;
    fn set_system_proxy_enabled(&mut self, enabled: bool) -> bool;

    fn proxy_request_logs(&mut self) -> String;
    fn lopxy_status(&mut self, timestamp: i64) -> String;
}

pub struct LopxyManagerServerStatus {
    controller: Arc<Mutex<dyn LopxyManagerServerController + Send>>
}

impl LopxyManagerServerStatus {
    pub fn new(controller: LopxyManagerServerControllerArc) -> LopxyManagerServerStatus {
        LopxyManagerServerStatus {
            controller
        }
    }
}

#[get("/shutdown")]
fn shutdown(state: &rocket::State<LopxyManagerServerStatus>, shutdown: rocket::Shutdown) -> &'static str {
    state.controller.lock().unwrap().shutdown();
    shutdown.notify();
    "Shutting down..."
}

#[get("/list")]
fn list_all_proxy_item(state: &rocket::State<LopxyManagerServerStatus>) -> Json<String> {
    Json(serde_json::to_string(state.controller.lock().unwrap().list_all_proxy_item()).unwrap_or("{}".to_string()))
}

#[post("/add", data = "<item>")]
fn add_proxy_item<'r>(state: &rocket::State<LopxyManagerServerStatus>, item: Form<AddRequest<'r>>) -> Json<String> {
    let resource_url = decode_url_string(item.resource_url);
    let proxy_resource_url = decode_url_string(item.proxy_resource_url);
    let resource_content_type = decode_url_string(item.resource_content_type);

    if resource_url.is_none() || proxy_resource_url.is_none() || resource_content_type.is_none() {
        return Json(String::from("{\"result\":false}"));
    }

    Json(format!("{{\"result\":{}}}", state.controller.lock().unwrap().add_proxy_item(&resource_url.unwrap(), &proxy_resource_url.unwrap(), &resource_content_type.unwrap())))
}

#[delete("/remove", data = "<item>")]
fn remove_proxy_item<'r>(state: &rocket::State<LopxyManagerServerStatus>, item: Form<RemoveRequest<'r>>) -> Json<String> {
    let resource_url = decode_url_string(item.resource_url);
    if resource_url.is_none() {
        return Json(String::from("{\"result\":false}"));
    }
    Json(format!("{{\"result\":{}}}", state.controller.lock().unwrap().remove_proxy_item(&resource_url.unwrap())))
}

#[post("/modify", data = "<item>")]
fn modify_proxy_item<'r>(state: &rocket::State<LopxyManagerServerStatus>, item: Form<ModifyRequest<'r>>) -> Json<String> {
    let resource_url = decode_url_string(item.resource_url);
    let proxy_resource_url = decode_url_string(item.proxy_resource_url);
    let resource_content_type = decode_url_string(item.resource_content_type);

    if resource_url.is_none() || proxy_resource_url.is_none() || resource_content_type.is_none() {
        return Json(String::from("{\"result\":false}"));
    }

    Json(format!("{{\"result\":{}}}", state.controller.lock().unwrap().modify_proxy_item(&resource_url.unwrap(), &proxy_resource_url.unwrap(), &resource_content_type.unwrap())))
}

#[get("/is_proxy_enabled")]
fn is_lopxy_proxy_enabled<'r>(state: &rocket::State<LopxyManagerServerStatus>) -> Json<String> {
    Json(format!("{{\"result\":{}}}", state.controller.lock().unwrap().is_system_proxy_enabled()))
}

#[post("/enable_proxy", data = "<item>")]
fn set_lopxy_proxy_enabled<'r>(state: &rocket::State<LopxyManagerServerStatus>, item: Form<SetProxyEnabledRequest>) -> Json<String> {
   Json(format!("{{\"result\":{}}}", state.controller.lock().unwrap().set_system_proxy_enabled(item.enabled)))
}

#[get("/proxy_request_logs")]
fn proxy_request_logs(state: &rocket::State<LopxyManagerServerStatus>) -> Json<String> {
    Json(state.controller.lock().unwrap().proxy_request_logs())
}

#[get("/status/<timestamp>")]
fn lopxy_status<'r>(state: &rocket::State<LopxyManagerServerStatus>, timestamp: i64) -> Json<String> {
    Json(state.controller.lock().unwrap().lopxy_status(timestamp))
}

#[catch(404)]
fn general_not_found() -> Html<&'static str> {
    Html(r#"
    <html>
        <body style="background-color: #e62b60">
            <center style="margin-top: 10%">
                <h1>404 NOT FOUND</h1>
                <p>lopxy</p>
            </center>
        </body>
    </html>
    "#)
}

///
/// Generate lopxy Web Manager Routes Vector
/// 
pub fn lopxy_web_manager_routes() -> Vec<rocket::Route> {
    routes![
        shutdown,
        list_all_proxy_item,
        add_proxy_item,
        remove_proxy_item,
        modify_proxy_item,
        is_lopxy_proxy_enabled,
        set_lopxy_proxy_enabled,
        proxy_request_logs,
        lopxy_status
    ]
}

///
/// Generate lopxy Web Manager Catcher Vector
/// 
pub fn lopxy_web_manager_catcher() -> Vec<rocket::Catcher> {
    catchers![
        general_not_found
    ]
}