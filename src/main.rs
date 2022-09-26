use clap::{Parser, Subcommand};
use sqlite::Value;
use std::error::Error;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Args {
    #[clap(subcommand)]
    commands: Commands,
}

// #[derive(Subcommand, Debug)]
// enum Commands {
//     Append {
//         #[clap(subcommand)]
//         details: Details,
//     },
//     Show {
//         #[clap(subcommand)]
//         details: Details,
//     },
//     Search {
//         text: String,
//     },
// }

// #[derive(Subcommand, Debug)]
// enum Details {
//     Phone { phone: String },
//     Job { job: String },
//     Notes { notes: String },
// }

#[derive(clap::Args, Debug)]
struct Details {
    #[clap(long)]
    name: String,
    #[clap(short, long)]
    job_title: Option<String>,
    #[clap(short, long)]
    company: Option<String>,
    #[clap(short, long)]
    notes: Vec<String>,
    #[clap(short, long)]
    disambiguation: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    New(Details),
    List,
    Edit,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let conn = sqlite::open("ppl.db")?;

    conn.execute(
        "
create table if not exists People (
ID integer primary key,
NAME text not null,
DISAMBIGUATION text, 
DATE_ADDED text not null
) ;

create table if not exists Companies (
ID integer primary key,
NAME text not null,
DESCRIPTION text,
DATE_ADDED text not null
) without rowid;

create table if not exists Job_titles (
ID integer primary key,
NAME text not null,
DESCRIPTION text,
DATE_ADDED text not null
) without rowid;

create table if not exists Notes (
ID integer primary key,
NOTE text not null,
DATE_ADDED text not null
) without rowid;

create table if not exists People_Notes (
ID integer primary key,
PEOPLE_ID integer not null,
NOTES_ID integer not null,
DATE_ADDED text not null,
foreign key (people_id) references People (ID) on update cascade on delete cascade,
foreign key (notes_id) references Notes (ID) on update cascade on delete cascade
) without rowid;

create table if not exists People_Companies (
ID integer primary key,
PEOPLE_ID integer not null,
COMPANIES_ID integer not null,
DATE_ADDED text not null,
foreign key (people_id) references People (ID) on update cascade on delete cascade,
foreign key (companies_id) references Companies (ID) on update cascade on delete cascade
) without rowid;

create table if not exists People_Job_titles (
ID integer primary key,
PEOPLE_ID integer not null,
JOB_TITLES_ID integer not null,
DATE_ADDED text not null,
foreign key (people_id) references People (ID) on update cascade on delete cascade,
foreign key (job_titles_id) references Job_titles (ID) on update cascade on delete cascade
) without rowid;
",
    )?;

    match args.commands {
        Commands::New(details) => {
            new(&details, &conn)?;
        }
        Commands::List => {
            let people = list(&conn)?;
            println! {"{:?}", people};
        }
        Commands::Edit => {}
    }

    Ok(())
}

fn new(details: &Details, c: &sqlite::Connection) -> Result<(), Box<dyn Error>> {
    let mut stmt = format! {
        "
INSERT INTO People (id, name, disambiguation, date_added)
VALUES ((select COUNT(id)+1 from People), '{}', {}, DATETIME());
",
        details.name,
        match &details.disambiguation {
            Some(d) => format!{"'{}'", d},
            None => format!{"null"},
        }
    };

    if let Some(company) = &details.company {
        stmt += &format! {"
INSERT INTO Companies (id, name, description, date_added)
VALUES ((select COUNT(id)+1 from Companies), '{}', null, DATETIME());
",
                          company,
        };

        stmt += "
    INSERT INTO People_Companies (id, people_id, companies_id, date_added)
    VALUES ((select COUNT(id)+1 from People_Companies), (select max(id) from People), (select max(id) from Companies), DATETIME());
    ";
    }

    if let Some(job_titles) = &details.job_title {
        stmt += &format! {"INSERT INTO Job_titles (id, name, description, date_added) VALUES ((select COUNT(id)+1 from Job_titles), '{}', null, DATETIME());", job_titles};
        stmt += "
    INSERT INTO People_Job_titles (id, people_id, job_titles_id, date_added)
    VALUES ((select COUNT(id)+1 from People_Job_titles), (select max(id) from People), (select max(id) from Job_titles), DATETIME());
    ";
    }

    for note in &details.notes {
        stmt += &format! {"INSERT INTO Notes (id, note, date_added) VALUES ((select COUNT(id)+1 from Notes), '{}', DATETIME());", note};
        stmt += "
    INSERT INTO People_Notes (id, people_id, notes_id, date_added)
    VALUES ((select COUNT(id)+1 from People_Notes), (select max(id) from People), (select max(id) from Notes), DATETIME());
    ";
    }

    c.execute(stmt)?;

    Ok(())
}

fn list(c: &sqlite::Connection) -> Result<Vec<String>, Box<dyn Error>> {
    let mut stmt = c
        .prepare("SELECT * FROM People ORDER BY DATE_ADDED ASC")?
        .into_cursor();

    let mut v = Vec::new();
    while let Some(Ok(row)) = stmt.next() {
        let name = row.get::<String, _>(1);
        v.push(name);
    }
    Ok(v)
}
