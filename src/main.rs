use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};
use std::io::{self, Read};

#[derive(Clone)]
struct Contestant {
    wins: i32,
    draws: i32,
    losses: i32,
    tiebreak: i32,
    history: HashSet<String>,
    name: String,
    byes: i32,
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
            tiebreak: 0,
            history: HashSet::new(),
            name: csv_name.to_string(),
            byes: 0,
            enabled: true,
        });
    }
    contestants.shuffle(&mut rand::thread_rng());

    for result in line_iter
        .filter(|x| x.len() > 0)
        .filter(|x| !x.starts_with("//"))
    {
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
                .expect(format!("Could not find contestant {}!", csvs[0]).as_str());
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
                .expect(format!("Could not find contestant {}!", csvs[2]).as_str());
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
                .expect(format!("Could not find contestant {}!", csvs[0]).as_str());
            con.wins += 1;
            con.byes += 1;
        }
    }

    // Only generate new games for the players with fewest played games. This finishes/starts the
    // current round
    let fewest_games = contestants
        .iter()
        .filter(|x| x.enabled == true)
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
        // We will find a match pairing for the player with the most amount of Byes.
        // Ties will be broken with the least options amongst available contestants.
        // Ties will be broken by having the highest score.
        let a_cons_set = available_cons
            .iter()
            .map(|x| x.name.to_string())
            .collect::<HashSet<String>>();
        let selected_con_index = available_cons
            .iter()
            .enumerate()
            .min_by(|a, b| {
                b.1.byes.cmp(&(a.1.byes)).then_with(|| {
                    a_cons_set
                        .difference(&a.1.history)
                        .count()
                        .cmp(&a_cons_set.difference(&b.1.history).count())
                        .then_with(|| (2 * b.1.wins + b.1.draws).cmp(&(2 * a.1.wins + a.1.draws)))
                })
            })
            .unwrap()
            .0;
        let selected_con = available_cons.swap_remove(selected_con_index);

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
            .expect(format!("No matching contestant available for {}. Rerun the program or revise the tournament history.", selected_con.name).as_str())
            .0;
        let matched_con = available_cons.swap_remove(matched_con_index);

        matchups.push((selected_con.name.to_string(), matched_con.name.to_string()));
    }

    calculate_tiebreaks(&mut contestants);
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
                tiebreak: 0,
                history: HashSet::new(),
                byes: 0,
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

fn calculate_tiebreaks(contestants: &mut Vec<Contestant>) {
    let score_map: HashMap<String, i32> = contestants
        .iter()
        .map(|x| (x.name.to_string(), 2 * x.wins + x.draws))
        .collect();

    for c in contestants.iter_mut() {
        c.tiebreak = c.history.iter().map(|x| score_map[x]).sum();
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

    if contestants.iter().any(|x| x.enabled == false) {
        println!("<h2>Disabled Contestants</h2>");
        print_results(&mut contestants, false);
    }

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
          <th>Tiebreak</th>
        </tr>
      </thead>
      <tbody>"#
    );

    contestants.sort_unstable_by(|a, b| {
        (2 * b.wins + b.draws)
            .cmp(&(2 * a.wins + a.draws))
            .then_with(|| (a.wins + a.draws + a.losses).cmp(&(b.wins + b.draws + b.losses)))
            .then_with(|| b.tiebreak.cmp(&a.tiebreak))
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
          <td>{:.2}</td>
        </tr>"#,
            c.name,
            ((2 * c.wins + c.draws) as f32) / 2.0,
            c.wins + c.draws + c.losses,
            c.wins,
            c.draws,
            c.losses,
            (c.tiebreak as f32 / (c.wins + c.draws + c.losses) as f32) / 2.0
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
