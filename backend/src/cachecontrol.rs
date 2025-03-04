use rocket::{
    fairing::*,
    http::{Header, hyper::header::CACHE_CONTROL},
};

pub struct Cacher;

#[rocket::async_trait]
impl Fairing for Cacher {
    fn info(&self) -> Info {
        Info {
            name: "Adds Cache Control",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r rocket::Request<'_>, res: &mut rocket::Response<'r>) {
        if let Some("assets") = req.routed_segment(0) {
            res.set_header(Header::new(CACHE_CONTROL.as_str(), "public, max-age=3600"));
        }
    }
}
