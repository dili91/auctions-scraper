extern crate log;

use failure::Error;
use headless_chrome::{Browser, Element, LaunchOptionsBuilder};
use log::*;
use serde::{Deserialize, Serialize};

const AUCTION_WEBSITE_URL: &str = "https://pvp.giustizia.it";
const AUCTION_RESULTS_PAGE: &str = "/pvp/en/risultati_ricerca.page";
const RESULTS_WINDOW_SIZE: i32 = 10;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Auction {
    location: String,
    batch: String,
    date_of_sale: String,
    minimum_bid_price: String,
    minimum_increment: String,
    procedure: String,
    starting_price: String,
    link: String,
}

fn main() {
    env_logger::init();
    let auctions = browse_auctions().unwrap();

    for a in auctions {
        println!("{}", serde_json::to_string_pretty(&a).unwrap());
    }
}

fn scrape_info_by_css_class(element: &Element, class: &str) -> String {
    match element
        .get_description()
        .unwrap()
        .find(|n| {
            n.attributes.as_ref().is_some()
                && n.attributes.as_ref().unwrap().get("class") == Some(&String::from(class))
        })
        .unwrap()
        .find(|n| n.node_name == "#text")
    {
        Some(e) => String::from(e.node_value.trim()),
        None => String::new(),
    }
}

fn scrape_info_by_text(element: &Element, text: &str) -> String {
    match element
        .get_description()
        .unwrap()
        .find(|n| {
            n.children.is_some()
                && n.children
                    .as_ref()
                    .unwrap()
                    .iter()
                    .any(|c| c.node_value.contains(text))
        })
        .unwrap()
        .children
        .as_ref()
        .unwrap()[1]
        .find(|n| n.node_name == "#text")
    {
        Some(e) => String::from(e.node_value.trim()),
        None => String::new(),
    }
}

fn parse_auction_item(element: &Element) -> Auction {
    let location = scrape_info_by_css_class(element, "anagrafica-risultato");

    let batch = scrape_info_by_css_class(element, "black");

    let date_of_sale = scrape_info_by_text(element, "Date of the judicial sale");

    let minimum_bid_price = scrape_info_by_text(element, "Minimum bid price");

    let minimum_increment = scrape_info_by_text(element, "Minimum increment");

    let procedure = scrape_info_by_text(element, "Procedure");

    let starting_price = scrape_info_by_text(element, "Starting price");

    let relative_link = match element.get_description().unwrap().find(|n| {
        n.attributes.as_ref().is_some() && n.attributes.as_ref().unwrap().contains_key("href")
    }) {
        Some(e) => String::from(e.attributes.as_ref().unwrap().get("href").unwrap()),
        None => String::new(),
    };

    let absolute_link = format!("{}{}", AUCTION_WEBSITE_URL, relative_link);

    Auction {
        location,
        batch,
        date_of_sale,
        minimum_bid_price,
        minimum_increment,
        procedure,
        starting_price,
        link: absolute_link,
    }
}

fn browse_auctions() -> Result<Vec<Auction>, Error> {
    let launch_opts = LaunchOptionsBuilder::default()
        .sandbox(false)
        .build()
        .unwrap();

    let browser = Browser::new(launch_opts)?;

    //let browser = Browser::default()?;

    let tab = browser.wait_for_initial_tab()?;

    let url = format!(
        "{}{}?elementiPerPagina={}",
        AUCTION_WEBSITE_URL, AUCTION_RESULTS_PAGE, RESULTS_WINDOW_SIZE
    );

    tab.navigate_to(&url)?;

    let mut auctions = Vec::<Auction>::new();

    match tab.wait_for_elements(".tile-dettaglio+ .col-xs-12") {
        Err(e) => error!("Unable to find auctions results: {:?}", e),
        Ok(items) => {
            for i in items.iter() {
                let auction = parse_auction_item(i);
                auctions.push(auction)
            }
        }
    }

    Ok(auctions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn it_works() {
        //todo: write tests
    }
}
