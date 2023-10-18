use actix_web::{get, web, App, HttpServer, Responder};

use rand::seq::SliceRandom;
use rand::thread_rng;

use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize)]
struct WebsiteName {
    website: String,
    count: Option<usize>,
}

#[derive(Serialize)]
struct WebsiteResponse {
    website_name: String,
    movies: Vec<Movie>,
}

#[derive(Serialize)]
struct Movie {
    title: String,
    year: String,
}

impl Clone for Movie {
    fn clone(&self) -> Self {
        Movie {
            title: self.title.clone(),
            year: self.year.clone(),
        }
    }
}

async fn get_html_from_website(website: String, count: Option<usize>) -> Vec<Movie> {
    let response = reqwest::get(website).await.unwrap();

    let html = response.text().await.unwrap_or("".to_string());

    let document = scraper::Html::parse_document(&html);

    let selector = scraper::Selector::parse("div.cli-children").unwrap();
    let title_selector = scraper::Selector::parse("a > h3").unwrap();
    let year_selector =
        scraper::Selector::parse("div.cli-title-metadata > span:nth-child(1)").unwrap();

    let mut movies: Vec<Movie> = Vec::new();

    for element in document.select(&selector) {
        let title = element.select(&title_selector).next().unwrap().inner_html();
        let year = element.select(&year_selector).next().unwrap().inner_html();
        movies.push(Movie { title, year });
    }

    movies.shuffle(&mut thread_rng());

    if count.is_some() {
        return movies[0..count.unwrap()].to_vec();
    }

    return movies;
}

#[get("/scrape")]
async fn hello(param: web::Query<WebsiteName>) -> impl Responder {
    format!("Welcome {}!", param.website);
    return web::Json(WebsiteResponse {
        website_name: param.website.clone(),
        movies: get_html_from_website(param.website.clone(), param.count).await,
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
