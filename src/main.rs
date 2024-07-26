use dialoguer::{
    console::{style, Style},
    theme::ColorfulTheme,
    Input, Select,
};
use crossterm::{terminal::{ClearType, Clear}, QueueableCommand, cursor::{MoveTo, Hide}};
use dotenv::dotenv;
use linebreak::LineIter;
use serde::Deserialize;
use std::{borrow::Borrow, io::{stdout, Result, Write}};
use urlencoding::encode;
use colored::Colorize;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Items {
    results: Vec<Item>,
}

#[derive(Deserialize, Debug)]
struct Item {
    id: u32,
    original_name: Option<String>,
    original_title: Option<String>,
    release_date: Option<String>,
    first_air_date: Option<String>,
    original_language: Option<String>,
    overview: Option<String>,
    media_type: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Details {
    id: u32,
    overview: Option<String>,
    original_language: Option<String>,
    title: Option<String>,
    media_type: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ItemDetails {
    number_of_episodes: Option<u32>,
    number_of_seasons: Option<u32>,
    tagline: Option<String>,
    runtime: Option<u32>,
}

impl Item {
    fn new(
        original_name: Option<String>,
        original_title: Option<String>,
        release_date: Option<String>,
        first_air_date: Option<String>,
        original_language: Option<String>,
        overview: Option<String>,
        media_type: Option<String>,
        id: u32,
    ) -> Self {
        Self {
            original_name,
            original_title,
            release_date,
            first_air_date,
            original_language,
            overview,
            media_type,
            id,
        }
    }
}

impl ItemDetails {
    fn new(
        number_of_episodes: Option<u32>,
        number_of_seasons: Option<u32>,
        tagline: Option<String>,
        runtime: Option<u32>,
    ) -> Self {
        Self {
            number_of_episodes,
            number_of_seasons,
            tagline,
            runtime
        }
    }
}

#[allow(dead_code)]
struct App {
    items: Items,
}

impl App {}

#[tokio::main]
async fn main() -> Result<()> {

    clear_screen();

    let search_string: String = Input::new()
        .with_prompt("  Enter your search term")
        .interact_text()
        .unwrap();

    let items = get_items(search_string.as_str()).await;
    let string_values = items.into_iter().map(|x| {
        (
            x.original_name.clone(),
            x.original_title.clone(),
            x.release_date.clone(),
            x.first_air_date.clone(),
            x.original_language.clone(),
            x.overview.clone(),
            x.media_type.clone(),
            x.id.clone(),
        )
    });

    let mut string_list: Vec<String> = Vec::new();
    let mut detail_list: Vec<Details> = Vec::new();

    for found in string_values {
        // original_name = serie or person
        if found.0.is_some() {
            if found.6.clone().unwrap() == "person" {
                string_list.push(format!(
                    "{} - {}",
                    found.0.clone().unwrap_or("".to_string()),
                    found.6.clone().unwrap_or("".to_string())
                ));
            } else {
                string_list.push(format!(
                    "{} - {} - released: {}",
                    found.0.clone().unwrap_or("".to_string()),
                    found.6.clone().unwrap_or("".to_string()),
                    found.3.unwrap_or("".to_string())
                ));
            }
            detail_list.push(Details {
                id: found.7,
                overview: Some(found.5.clone().unwrap_or("".to_string())),
                title: Some(found.0.clone().unwrap_or("".to_string())),
                original_language: Some(found.4.clone().unwrap_or("".to_string())),
                media_type: Some(found.6.clone().unwrap_or("".to_string())),
            });
        }
        // original_title = movie
        if found.1.is_some() {
            string_list.push(format!(
                "{} - {} - released: {}",
                found.1.clone().unwrap_or("".to_string()),
                found.6.clone().unwrap_or("".to_string()),
                found.2.unwrap_or("".to_string())
            ));
            detail_list.push(Details {
                id: found.7,
                overview: Some(found.5.clone().unwrap_or("".to_string())),
                title: Some(found.1.clone().unwrap_or("".to_string())),
                original_language: Some(found.4.clone().unwrap_or("".to_string())),
                media_type: Some(found.6.clone().unwrap_or("".to_string())),
            });
        }
    }

    let mut theme = ColorfulTheme::default();

    // Apply some custom styles
    theme.active_item_prefix = style("› ".to_string()).for_stderr().green();
    theme.active_item_style = Style::new().for_stderr().green();

    let selection = Select::with_theme(&theme)
        .with_prompt("Found results:")
        .default(0)
        .items(&string_list)
        .interact()
        .unwrap();

    let selected = &string_list[selection];
    let index = string_list.iter().position(|r| r == selected).unwrap();

    // Get more details about the selected item
    let search_id = detail_list[index].id.clone();
    let media_search_type = detail_list[index].media_type.as_mut().unwrap().as_str();

    let additional_details = get_details(search_id, media_search_type).await;

    println!("");
    println!("  {}", "Title:".green());
    println!("");
    println!("  {}", detail_list[index].title.clone().unwrap());
    println!("");

    // println!("The ID is:");
    // println!("{}", detail_list[index].id);

    println!("  {} {}",
        "Original language:".green(),
        detail_list[index]
            .original_language
            .clone()
            .unwrap()
            .as_str()
            .to_uppercase()
    );
    println!("");

    let overview = detail_list[index].overview.clone().unwrap();
    let mut overview_iter = LineIter::new(&overview, 80);
    overview_iter.set_indent("  ");

    println!("  {}", "Overview:".green());
    println!("");

    while let Some(line) = overview_iter.next() {
        println!("{}", line);
    }

    println!("");

    for values in additional_details {
        if detail_list[index].media_type.as_mut().unwrap().as_str() == "tv" {
            println!("  {} {}", "Number of seasons:".green(), values.number_of_seasons.as_ref().unwrap().to_string());
            println!("");
            println!("  {} {}", "Number of episodes:".green(), values.number_of_episodes.as_ref().unwrap().to_string());
            println!("");
            println!("{}", "  Tagline:".green());
            println!("");
            println!("  {}", values.tagline.clone().unwrap_or("".to_string()));
            println!("");
            println!("{}", "  For more details, visit:".green());
            println!("");
            println!("  https://www.themoviedb.org/tv/{}", detail_list[index].id.clone());
            println!("");
        } else {
            println!("  {}", "Tagline:".green());
            println!("");
            println!("  {}", values.tagline.clone().unwrap_or("".to_string()));
            println!("");
            println!("  {}", "Runtime:".green());
            println!("");
            println!("  {} mins.", values.runtime.clone().unwrap_or(0));
            println!("");
            println!("  {}", "For more details, visit:".green());
            println!("");
            println!("  https://www.themoviedb.org/movie/{}", detail_list[index].id.clone());
            println!("");
        }
    }

    Ok(())
}

async fn get_items(term: &str) -> Vec<Item> {
    dotenv().ok();

    let mut items: Vec<Item> = Vec::new();

    let bearer_token = std::env::var("TMDB_TOKEN").expect("TMDB_TOKEN must be set.");
    let base_url: &str = "https://api.themoviedb.org/3/search/multi";
    let params = "?page=1&include_adult=false&language=en-US&page=1&query=";
    let search_term = encode(&term);
    let complete_url = format!("{base_url}{params}{search_term}");

    let client: reqwest::Client = reqwest::Client::new();
    let response = client
        .get(complete_url)
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer ") + &bearer_token)
        .send()
        .await;

    match response {
        Ok(data) => {
            let data_items = data.json::<Items>().await;
            for stuff in data_items.borrow() {
                for item in stuff.results.as_slice() {
                    items.push(Item::new(
                        item.original_name.clone(),
                        item.original_title.clone(),
                        item.release_date.clone(),
                        item.first_air_date.clone(),
                        item.original_language.clone(),
                        item.overview.clone(),
                        item.media_type.clone(),
                        item.id,
                    ));
                }
            }
        }
        Err(err) => {
            println!("Something went wrong {:?}", err);
        }
    }

    return items;
}

async fn get_details(id: u32, media_type: &str) -> Vec<ItemDetails> {
    dotenv().ok();

    let mut item: Vec<ItemDetails> = Vec::new();

    let bearer_token = std::env::var("TMDB_TOKEN").expect("TMDB_TOKEN must be set.");
    let base_url: &str = "https://api.themoviedb.org/3/";
    let params = "?language=en-US";
    let complete_url = format!("{base_url}{media_type}/{id}{params}");

    let client: reqwest::Client = reqwest::Client::new();
    let response = client
        .get(complete_url)
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer ") + &bearer_token)
        .send()
        .await;

    match response {
        Ok(data) => {
            let data_items = data.json::<ItemDetails>().await;

            for value in data_items.borrow() {
                item.push(ItemDetails::new(
                    value.number_of_episodes.clone(),
                    value.number_of_seasons.clone(),
                    value.tagline.clone(),
                    value.runtime.clone()
                ));
            }
        }
        Err(err) => {
            println!("Something went wrong {:?}", err);
        }
    }

    return item;
}

pub fn clear_screen() {
    let mut out = stdout();
    out.queue(Hide).unwrap();
    out.queue(Clear(ClearType::All)).unwrap();
    out.queue(MoveTo(0, 0)).unwrap();
    out.flush().unwrap();
}