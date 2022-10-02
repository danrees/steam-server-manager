use rocket::response::stream::{Event, EventStream};

#[get("/events")]
pub async fn test_events() -> EventStream![] {
    EventStream! {
        let mut interval = rocket::tokio::time::interval(rocket::tokio::time::Duration::from_secs(1));
        loop {
            yield Event::data("ping");
            interval.tick().await;
        }
    }
}
