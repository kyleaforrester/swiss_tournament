use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::io::{self, Read};

#[derive(Clone)]
struct Contestant {
    wins: i32,
    draws: i32,
    losses: i32,
    history: HashSet<String>,
    name: String,
    enabled: bool,
}

fn main() {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Could not read from stdin.");
    let mut line_iter = buffer.split('\n');

    let mut contestants: Vec<Contestant> = Vec::new();
    for csv_name in line_iter.next().unwrap().split(',').map(|x| x.trim()) {
        contestants.push(Contestant {
            wins: 0,
            draws: 0,
            losses: 0,
            history: HashSet::new(),
            name: csv_name.to_string(),
            enabled: true,
        });
    }
    contestants.shuffle(&mut rand::thread_rng());

    for result in line_iter.filter(|x| x.len() > 0) {
        if result.chars().nth(0).unwrap() == '#' {
            execute_command(result, &mut contestants);
            continue;
        }

        let csvs: Vec<&str> = result.split(',').collect();

        if csvs.len() == 4 {
            let mut con = contestants
                .iter_mut()
                .filter(|x| x.name == csvs[0])
                .nth(0)
                .unwrap();
            match csvs[1] {
                "W" => con.wins += 1,
                "D" => con.draws += 1,
                "L" => con.losses += 1,
                _ => panic!("Unrecognized game result: {}", csvs[1]),
            }
            con.history.insert(csvs[2].to_string());

            let mut con = contestants
                .iter_mut()
                .filter(|x| x.name == csvs[2])
                .nth(0)
                .unwrap();
            match csvs[3] {
                "W" => con.wins += 1,
                "D" => con.draws += 1,
                "L" => con.losses += 1,
                _ => panic!("Unrecognized game result: {}", csvs[3]),
            }
            con.history.insert(csvs[0].to_string());
        } else if csvs.len() == 2 {
            let mut con = contestants
                .iter_mut()
                .filter(|x| x.name == csvs[0])
                .nth(0)
                .unwrap();
            con.wins += 1;
        }
    }

    // Only generate new games for the players with fewest played games. This finishes/starts the
    // current round
    let fewest_games = contestants
        .iter()
        .map(|x| x.wins + x.draws + x.losses)
        .min()
        .unwrap();
    let mut matchups: Vec<(String, String)> = Vec::new();
    let mut available_cons: Vec<Contestant> = contestants
        .iter()
        .filter(|x| x.wins + x.draws + x.losses == fewest_games && x.enabled == true)
        .map(|x| x.clone())
        .collect();

    while available_cons.len() > 1 {
        // We will find a match pairing for the player with the least options amongst available contestants.
        // Ties will be broken by having the highest score.
        let a_cons_set = available_cons
            .iter()
            .map(|x| x.name.to_string())
            .collect::<HashSet<String>>();
        let selected_con_index = available_cons
            .iter()
            .enumerate()
            .min_by(|a, b| {
                a_cons_set
                    .difference(&a.1.history)
                    .count()
                    .cmp(&a_cons_set.difference(&b.1.history).count())
                    .then_with(|| (2 * b.1.wins + b.1.draws).cmp(&(2 * a.1.wins + a.1.draws)))
            })
            .unwrap()
            .0;
        let selected_con = available_cons.swap_remove(selected_con_index);

        /*
        println!("\nSearching for match for {}", selected_con.name);
        println!("Matchups: {:?}", matchups);
        println!(
            "All Available: {:?}",
            available_cons
                .iter()
                .map(|x| x.name.as_str())
                .collect::<Vec<&str>>()
        );
        println!(
            "New Available: {:?}",
            available_cons
                .iter()
                .filter(|x| !selected_con.history.contains(&x.name))
                .map(|x| x.name.as_str())
                .collect::<Vec<&str>>()
        );
        */

        // Our match will be a player we have not played with the closest score to our own
        // Ties will be broken by having the least options amongst the available contestants.
        let matched_con_index = available_cons
            .iter()
            .enumerate()
            .filter(|x| !selected_con.history.contains(&x.1.name))
            .min_by(|a, b| {
                ((2 * selected_con.wins + selected_con.draws) - (2 * a.1.wins + a.1.draws))
                    .abs()
                    .cmp(
                        &((2 * selected_con.wins + selected_con.draws)
                            - (2 * b.1.wins + b.1.draws))
                            .abs(),
                    )
                    .then_with(|| {
                        a_cons_set
                            .difference(&a.1.history)
                            .count()
                            .cmp(&(a_cons_set.difference(&b.1.history).count()))
                    })
            })
            .unwrap()
            .0;
        let matched_con = available_cons.swap_remove(matched_con_index);

        matchups.push((selected_con.name.to_string(), matched_con.name.to_string()));
    }

    print_output(contestants, matchups, available_cons);
}

fn execute_command(command: &str, contestants: &mut Vec<Contestant>) {
    match command.split(' ').nth(0).unwrap() {
        "#add" => {
            let mut char_data_iter = command.split_once(' ').unwrap().1.split(',');
            contestants.push(Contestant {
                name: char_data_iter.next().unwrap().to_string(),
                wins: char_data_iter.next().unwrap().parse().unwrap(),
                draws: char_data_iter.next().unwrap().parse().unwrap(),
                losses: char_data_iter.next().unwrap().parse().unwrap(),
                history: HashSet::new(),
                enabled: true,
            });
        }
        "#disable" => {
            let name = command.split_once(' ').unwrap().1;
            contestants
                .iter_mut()
                .find(|x| x.name == name)
                .unwrap()
                .enabled = false;
        }
        "#enable" => {
            let mut char_data_iter = command.split_once(' ').unwrap().1.split(',');
            let name = char_data_iter.next().unwrap();
            let mut contestant = contestants
                .iter_mut()
                .filter(|x| x.enabled == false)
                .find(|x| x.name == name)
                .unwrap();
            contestant.wins = char_data_iter.next().unwrap().parse().unwrap();
            contestant.draws = char_data_iter.next().unwrap().parse().unwrap();
            contestant.losses = char_data_iter.next().unwrap().parse().unwrap();
            contestant.enabled = true;
        }
        _ => panic!("Unknown command: {}", command.split(' ').nth(0).unwrap()),
    }
}

fn print_output(
    mut contestants: Vec<Contestant>,
    matchups: Vec<(String, String)>,
    available_cons: Vec<Contestant>,
) {
    println!(
        r#"<!DOCTYPE html>
<html>
  <head>
    <title>Swiss-System Tournament Results</title>
  </head>
  <body>
    <h1>Swiss-System Tournament Results</h1>"#
    );

    print_results(&mut contestants, true);
    print_matchups(matchups, available_cons);

    println!("<h2>Disabled Contestants</h2>");
    print_results(&mut contestants, false);

    println!("  </body>\n</html>");
}

fn print_results(contestants: &mut Vec<Contestant>, enabled: bool) {
    println!(
        r#"    <table>
      <thead>
        <tr>
          <th>Player Name</th>
          <th>Score</th>
          <th>Games Played</th>
          <th>Wins</th>
          <th>Draws</th>
          <th>Losses</th>
        </tr>
      </thead>
      <tbody>"#
    );

    contestants.sort_unstable_by(|a, b| {
        (2 * b.wins + b.draws)
            .cmp(&(2 * a.wins + a.draws))
            .then_with(|| (a.wins + a.draws + a.losses).cmp(&(b.wins + b.draws + b.losses)))
    });
    for c in contestants.iter().filter(|x| x.enabled == enabled) {
        println!(
            r#"        <tr>
          <td>{}</td>
          <td>{:.1}</td>
          <td>{}</td>
          <td>{}</td>
          <td>{}</td>
          <td>{}</td>
        </tr>"#,
            c.name,
            ((2 * c.wins + c.draws) as f32) / 2.0,
            c.wins + c.draws + c.losses,
            c.wins,
            c.draws,
            c.losses
        );
    }

    println!("      </tbody>\n    </table>");
}

fn print_matchups(matchups: Vec<(String, String)>, available_cons: Vec<Contestant>) {
    println!("    <h2>Next Round Matchups</h2>\n    <ul>");

    for m in matchups.iter() {
        println!("      <li>{} vs. {}</li>", m.0, m.1);
    }

    for c in available_cons.iter() {
        println!("      <li>{} gets a BYE</li>", c.name);
    }

    println!("    </ul>");
}
