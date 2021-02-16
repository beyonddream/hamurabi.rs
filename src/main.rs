use rand::{rngs::ThreadRng, Rng};
use std::process::exit;
use text_io::try_scan;

#[derive(Clone, Copy)]
enum CityEvent {
    Plague,
    BuyAcres,
    SellAcres,
    FeedPeople,
    PlantSeeds,
    HarvestBounty,
    None,
    Exit,
}

#[derive(Clone, Copy)]
enum PlayerScore {
    Worse,
    Bad,
    Fair,
    Good,
}

#[derive(Clone, Copy)]
struct City {
    year: u16,
    bushels_preserved: u16,
    bushels_destroyed: u16,
    bushels_per_acre: u16,
    people_starved: u16,
    people_arrived: u16,
    population: u16,
    acres_owned: u16,
    acres_planted_with_seed: u16,
}

fn get_new_city() -> City {
    let city = City {
        year: 1,
        bushels_preserved: 2800,
        bushels_destroyed: 200,
        bushels_per_acre: 3,
        people_starved: 0,
        people_arrived: 5,
        population: 95,
        acres_owned: 0, // dependent on other fields on the struct
        acres_planted_with_seed: 0,
    };

    City {
        acres_owned: (city.bushels_preserved + city.bushels_destroyed) / city.bushels_per_acre,
        ..city
    }
}

fn check_plague(acres_buy_or_sell: u16, city: &mut City) -> CityEvent {
    // original game logic - assume there is a plague if no acres are bought or sold.
    if acres_buy_or_sell <= 0 {
        city.population = city.population / 2;
        return CityEvent::Plague;
    }
    CityEvent::None
}

fn update_report_summary(city: &mut City, acres_buy_or_sell: &u16) {
    println!(
        "Hamurabu: I beg to report to you, In year {}, {} people starved, {} came to the city.",
        city.year, city.people_starved, city.people_arrived
    );

    city.year += 1;
    city.population += city.people_arrived;

    if let CityEvent::Plague = check_plague(*acres_buy_or_sell, city) {
        println!("A horrible plague struck! Half the people died.");
    }

    println!("Population is now {}", city.population);
    println!("The city owns {} acres.", city.acres_owned);
    println!(
        "You have harvested {} bushels per acre.",
        city.bushels_per_acre
    );
    println!("Rats ate {} bushels.", city.bushels_destroyed);
    println!("You now have {} bushels in store.", city.bushels_preserved);
}

fn game_result(city: &City, population_starved_per_yr: u16, people_died_total: u16) -> PlayerScore {
    let acres_per_person: u16 = city.acres_owned / city.population;

    println!(
        "In your 10-yr term of office, {} percent of the population starved per \
        year on average, i.e., A total of {} people died!! You started with 10 acres \
        per person and ended with {} acres per person.",
        population_starved_per_yr, people_died_total, acres_per_person
    );

    if population_starved_per_yr > 33 || acres_per_person < 7 {
        return PlayerScore::Worse;
    } else if population_starved_per_yr > 10 || acres_per_person < 9 {
        return PlayerScore::Bad;
    } else if population_starved_per_yr > 3 || acres_per_person < 10 {
        return PlayerScore::Fair;
    }

    PlayerScore::Good
}

fn game_illegal_input() {
    println!("Hamurabi: I cannot do what you wish. Get yourself another steward!!!!!");
}

fn game_illegal_bushels_input(bushels_total: u16) {
    print!(
        "Hamurabi: Think again. You have only {} bushels of grain. Now then, ",
        bushels_total
    );
}

fn game_illegal_acres_input(acres_total: u16) {
    print!(
        "Hamurabi: Think again. You own only {} acres. Now then, ",
        acres_total
    );
}

fn game_get_user_input_validated() -> Result<i32, text_io::Error> {
    let user_input: i32;
    try_scan!("{}", user_input);

    Ok(user_input)
}

fn game_get_user_input(user_prompt: &'static str) -> i32 {
    print!("{} (input a number >= 0)", user_prompt);

    match game_get_user_input_validated() {
        Ok(user_input) => user_input,
        Err(_) => {
            game_illegal_input();
            exit(1); /* Game end */
        }
    }
}

fn game_get_random_event_value(multiplier: f64, padding: i64) -> i64 {
    let mut rng = rand::thread_rng();
    let random_event_value = rng.gen::<f64>();

    (random_event_value * multiplier) as i64 + padding
}

fn check_rat_menace(city: &mut City) {
    let random_event_value;

    city.bushels_destroyed = 0;
    random_event_value = game_get_random_event_value(5.0, 1);

    if (random_event_value as f64 / 2.0) as i64 == (random_event_value / 2) {
        city.bushels_destroyed = city.bushels_preserved / random_event_value as u16;
    }
}

fn harvest_bounty(
    city: &mut City,
    acres_buy_or_sell: &mut u16,
    population_starved_per_yr: &mut u16,
    people_died_total: &mut u16,
) -> CityEvent {
    let new_bushels;

    let random_event_value = game_get_random_event_value(5.0, 1);
    city.bushels_per_acre = random_event_value as u16;
    new_bushels = city.acres_planted_with_seed * city.bushels_per_acre;

    check_rat_menace(city);

    city.bushels_preserved += new_bushels - city.bushels_destroyed;
    let random_event_value = game_get_random_event_value(5.0, 1);
    city.people_arrived = (random_event_value
        * ((20 * city.acres_owned) + city.bushels_preserved) as i64
        / city.population as i64
        / 101 as i64) as u16;

    let random_event_value = *acres_buy_or_sell / 20;
    *acres_buy_or_sell = 10 * ((2 * game_get_random_event_value(1.0, 0)) as f64 - 0.3) as u16;

    if city.population < random_event_value {
        city.people_starved = 0;
        return CityEvent::None;
    }

    city.people_starved = city.population - random_event_value;

    if city.people_starved > (0.45 * city.population as f64) as u16 {
        println!("You starved {} people in one year", city.people_starved);
        game_print_result_worse();
        return CityEvent::Exit;
    }

    *population_starved_per_yr = (((city.year - 1) * (*population_starved_per_yr))
        + (city.people_starved * 100 / city.population))
        / city.year;
    city.population = random_event_value;
    *people_died_total += city.people_starved;

    CityEvent::None
}

fn plant_seeds(city: &mut City) -> CityEvent {
    let acres_to_plant: &mut u16 = &mut city.people_starved;

    loop {
        *acres_to_plant =
            game_get_user_input("How many acres do you wish to plant with seed?") as u16;

        if *acres_to_plant == 0 {
            city.acres_planted_with_seed = *acres_to_plant;
            return CityEvent::HarvestBounty;
        }

        if *acres_to_plant > city.acres_owned {
            game_illegal_acres_input(city.acres_owned);
            continue;
        } else if *acres_to_plant / 2 >= city.bushels_preserved {
            game_illegal_bushels_input(city.bushels_preserved);
            continue;
        } else if *acres_to_plant >= (10 * city.population) {
            print!(
                "But you have only {} people to tend the fields. Now then, ",
                city.population
            );
            continue;
        }
        break;
    }

    city.acres_planted_with_seed = *acres_to_plant;
    city.bushels_preserved -= *acres_to_plant / 2;

    CityEvent::HarvestBounty
}

fn feed_people(city: &mut City, acres_buy_or_sell: &mut u16) -> CityEvent {
    let bushels_to_feed_people = acres_buy_or_sell;
    let bushels_preserved = city.bushels_preserved;

    loop {
        *bushels_to_feed_people =
            game_get_user_input("How many bushels do you wish to feed your people?") as u16;

        if *bushels_to_feed_people > bushels_preserved {
            game_illegal_bushels_input(bushels_preserved);
            continue;
        }
        break;
    }

    city.bushels_preserved -= *bushels_to_feed_people;

    CityEvent::PlantSeeds
}

fn sell_acres(city: &mut City, acres_buy_or_sell: &mut u16) -> CityEvent {
    let bushels_per_acre: u16 = city.bushels_per_acre;

    loop {
        *acres_buy_or_sell = game_get_user_input("How many acres do you wish to sell?") as u16;

        if *acres_buy_or_sell >= city.acres_owned {
            game_illegal_acres_input(city.acres_owned);
            continue;
        }
        break;
    }

    city.acres_owned -= *acres_buy_or_sell;
    city.bushels_preserved += bushels_per_acre * (*acres_buy_or_sell);

    CityEvent::FeedPeople
}

fn buy_acres(city: &mut City, acres_buy_or_sell: &mut u16) -> CityEvent {
    let bushels_preserved = city.bushels_preserved;
    let bushels_per_acre = city.bushels_per_acre;

    loop {
        *acres_buy_or_sell = game_get_user_input("How many acres do you wish to buy?") as u16;

        if bushels_per_acre * (*acres_buy_or_sell) > bushels_preserved {
            game_illegal_bushels_input(bushels_preserved);
            continue;
        }
        break;
    }

    if *acres_buy_or_sell != 0 {
        city.acres_owned += *acres_buy_or_sell;
        city.bushels_preserved -= bushels_per_acre * (*acres_buy_or_sell);
        return CityEvent::FeedPeople;
    }

    return CityEvent::SellAcres;
}

fn game_print_result_worse() {
    println!("Due to this extreme mismanagement you have not only been impeached \
                                                and thrown out of office but you have also been declared 'NATIONAL FINK'!!");
}

fn game_print_result_bad() {
    println!("Your heavy handed performance smacks of Nero and Ivan IV. The people \
                                                (remaining) find you an unpleasant ruler, and, frankly, hate your guts!");
}

fn game_print_result_fair(city: &City, rng: &mut ThreadRng) {
    let rebels = (city.population * rng.gen::<u16>()) as f64 * 0.8;
    println!("Your performance could have been somewhat better, but really wasn't too bad at all. \
                            {} people would dearly like to see you assasinated but we all have our trivial problems.", rebels);
}

fn game_print_result_good() {
    println!(
        "A fantastic performance!!! Charlemange, Disraeli, and Jefferson combined \
                                                could not have done better!"
    );
}

fn game_start() {
    let mut acres_buy_or_sell: u16;
    let mut population_starved_per_yr: u16;
    let mut people_died_total: u16;
    let mut rng = rand::thread_rng();

    println!(
        "Try your hand at governing ancient sumeria, successfully for a 10-yr term of office.\n"
    );

    let mut city = get_new_city();
    acres_buy_or_sell = 1;
    people_died_total = 0;
    population_starved_per_yr = 0;

    loop {
        update_report_summary(&mut city, &acres_buy_or_sell);

        if city.year > 10 {
            match game_result(&city, population_starved_per_yr, people_died_total) {
                PlayerScore::Worse => game_print_result_worse(),
                PlayerScore::Bad => game_print_result_bad(),
                PlayerScore::Fair => game_print_result_fair(&city, &mut rng),
                PlayerScore::Good => game_print_result_good(),
            }
            exit(0); /* Game end */
        } else {
            let random_event_value = rng.gen_range(0..10);
            city.bushels_per_acre = random_event_value + 17;
            println!(
                "Land is trading at {} bushels per acre.",
                city.bushels_per_acre
            );

            let mut game_state = CityEvent::BuyAcres;
            loop {
                match game_state {
                    CityEvent::BuyAcres => {
                        game_state = buy_acres(&mut city, &mut acres_buy_or_sell)
                    }
                    CityEvent::FeedPeople => {
                        game_state = feed_people(&mut city, &mut acres_buy_or_sell)
                    }
                    CityEvent::SellAcres => {
                        game_state = sell_acres(&mut city, &mut acres_buy_or_sell)
                    }
                    CityEvent::HarvestBounty => {
                        game_state = harvest_bounty(
                            &mut city,
                            &mut acres_buy_or_sell,
                            &mut population_starved_per_yr,
                            &mut people_died_total,
                        )
                    }
                    CityEvent::PlantSeeds => game_state = plant_seeds(&mut city),
                    CityEvent::Exit => exit(0), /* Game end */
                    _ => break,                 /* current year complete */
                }
            }
        }
    }
}

fn main() {
    game_start();
}
