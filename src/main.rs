#![feature(arc_unwrap_or_clone)]

use ao3_rs::{
    language::Language,
    search::{SearchQuery, SortBy, SortDirection},
    work::{Category, Rating, Warning, Work},
};
use strum::IntoEnumIterator;
// use crossterm::{
//     event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
//     execute,
//     terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
// };
use cursive::{
    align::HAlign,
    event::{Event, Key},
    theme,
    traits::{Nameable, Resizable, Scrollable},
    view::SizeConstraint,
    views::{
        Checkbox, Dialog, EditView, EnableableView, FixedLayout, LinearLayout, ListView,
        PaddedView, RadioButton, RadioGroup, ResizedView, ScrollView, SelectView, TextView,
    },
    View,
};
use std::{collections::HashSet, io, ops::Range, rc::Rc};
// use tui::{
//     backend::{Backend, CrosstermBackend},
//     buffer::Buffer,
//     layout::{Constraint, Direction, Layout, Rect},
//     style::Style,
//     text::{Span, Spans},
//     widgets::{Block, Borders, Paragraph, Widget},
//     Frame, Terminal,
// };

// struct TextBody {
//     text: String,
//     scroll_offset: usize,
// }

const VERT_CHARS: usize = 80;

// impl Widget for TextBody {
//     fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
//         let size = area.width.min(VERT_CHARS as u16);
//         let coords = Rect::new(area.width / 2 - size / 2, 0, size, area.height);

//         let processed_text = self
//             .text
//             .split("\n")
//             .map(|x| vec![x, " "])
//             .flatten()
//             .map(|x| {
//                 let chunks = x.chars().collect::<Vec<char>>();
//                 let mut chunks = chunks.chunks(size as usize);

//                 let mut v: Vec<String> = vec![];

//                 while let Some(chunk) = chunks.next() {
//                     v.push(chunk.iter().collect());
//                 }

//                 v
//             })
//             .flatten()
//             .map(|s| s.trim().to_string())
//             .skip(self.scroll_offset)
//             .take(area.height as usize)
//             .enumerate();

//         for (n, line) in processed_text {
//             buf.set_string(coords.x, coords.y + n as u16, line, Style::default())
//         }
//     }
// }

// struct App {
//     work: Option<Work>,
//     scroll_offset: usize,
// }

// impl App {
//     fn new() -> Self {
//         Self {
//             work: None,
//             scroll_offset: 0,
//         }
//     }
// }

// fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
//     let block = TextBody {
//         text: app.work.clone().unwrap().chapters[0].body.clone(),
//         scroll_offset: app.scroll_offset,
//     };

//     f.render_widget(block, f.size())
// }

// fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
//     app.work = Some(Work::from_id(24123610));

//     loop {
//         terminal.draw(|f| ui(f, &app))?;

//         if let Event::Key(key) = event::read()? {
//             match key.code {
//                 KeyCode::Char('q') => {
//                     return Ok(());
//                 }
//                 KeyCode::Down => match app.scroll_offset.checked_add(1) {
//                     Some(i) => app.scroll_offset = i,
//                     None => {}
//                 },
//                 KeyCode::Up => match app.scroll_offset.checked_sub(1) {
//                     Some(i) => app.scroll_offset = i,
//                     None => {}
//                 },
//                 _ => {}
//             }
//         }
//     }
// }

const MARGIN: usize = 1;
const NOTES_MARGIN: usize = VERT_CHARS / 8;

#[derive(Clone)]
struct App {
    chapter: usize,
    work: Work,
}

impl View for App {
    fn draw(&self, _: &cursive::Printer) {}
}

fn render_work(work: &Work, chapter: usize) -> LinearLayout {
    let processed_text = if chapter == 0 {
        work.metadata.summary.clone().replace("\n", "\n\n")
    } else {
        work.chapters[chapter - 1]
            .body
            .clone()
            .replace("\n", "\n\n")
    };

    let work_title = work.metadata.title.clone();

    let chapter_title = if chapter == 0 {
        String::new()
    } else {
        work.chapters[chapter - 1].title.clone().unwrap()
    };

    let chapter_number = format!(
        "{}/{}/{}",
        chapter,
        work.metadata.chapters.current,
        work.metadata
            .chapters
            .planned
            .map(|p| p.to_string())
            .unwrap_or("?".to_string())
    );

    let start_notes = if chapter == 0 {
        Some(String::new())
    } else {
        work.chapters[chapter - 1]
            .start_notes
            .clone()
            .map(|x| x.replace("\n", "\n\n"))
    };

    let end_notes = if chapter == 0 {
        work.start_notes.clone().map(|x| x.replace("\n", "\n\n"))
    } else {
        work.chapters[chapter - 1]
            .end_notes
            .clone()
            .map(|x| x.replace("\n", "\n\n"))
    };

    LinearLayout::vertical()
        .child(ResizedView::with_max_width(
            VERT_CHARS,
            PaddedView::lrtb(
                0,
                0,
                MARGIN,
                MARGIN,
                LinearLayout::horizontal()
                    .child(ResizedView::with_full_width(
                        TextView::new(work_title).h_align(HAlign::Center),
                    ))
                    .child(ResizedView::with_full_width(
                        TextView::new(chapter_title).h_align(HAlign::Center),
                    ))
                    .child(ResizedView::with_full_width(
                        TextView::new(chapter_number).h_align(HAlign::Center),
                    )),
            ),
        ))
        .child(ResizedView::with_max_width(
            VERT_CHARS,
            LinearLayout::vertical()
                .child(PaddedView::lrtb(
                    NOTES_MARGIN,
                    NOTES_MARGIN,
                    MARGIN,
                    0,
                    EnableableView::new(
                        TextView::new(start_notes.clone().unwrap_or(String::new()))
                            .h_align(HAlign::Center),
                    )
                    .with_enabled(start_notes.is_some()),
                ))
                .child(TextView::new(processed_text))
                .child(PaddedView::lrtb(
                    NOTES_MARGIN,
                    NOTES_MARGIN,
                    MARGIN,
                    0,
                    EnableableView::new(
                        TextView::new(end_notes.clone().unwrap_or(String::new()))
                            .h_align(HAlign::Center),
                    )
                    .with_enabled(end_notes.is_some()),
                ))
                .scrollable(),
        ))
}

fn main() -> Result<(), io::Error> {
    // Creates the cursive root - required for every application.
    let mut siv = cursive::default();
    siv.load_toml(include_str!("../theme.toml")).unwrap();

    // let work = Work::from_id(24123610);

    // siv.add_layer(
    //     App {
    //         work: work.clone(),
    //         chapter: 0,
    //     }
    //     .with_name("app"),
    // );

    let mut compl_group = RadioGroup::new();

    let compl_none_button = compl_group.button(None, "all");
    let compl_true_button = compl_group.button(Some(true), "complete");
    let compl_false_button = compl_group.button(Some(false), "in progress");

    let mut cross_group = RadioGroup::new();

    let cross_none_button = cross_group.button(None, "include");
    let cross_true_button = cross_group.button(Some(true), "only");
    let cross_false_button = cross_group.button(Some(false), "exclude");

    siv.add_layer(
        Dialog::new()
            .padding_lrtb(1, 1, 1, 0)
            .content(
                LinearLayout::vertical()
                    .child(TextView::new("any field"))
                    .child(EditView::new().with_name("any_field"))
                    .child(TextView::new("title"))
                    .child(EditView::new().with_name("title"))
                    .child(TextView::new("author"))
                    .child(EditView::new().with_name("author"))
                    .child(TextView::new("date"))
                    .child(EditView::new().with_name("date"))
                    .child(TextView::new("completion"))
                    .child(compl_none_button)
                    .child(compl_true_button)
                    .child(compl_false_button)
                    .child(TextView::new("crossovers"))
                    .child(cross_none_button)
                    .child(cross_true_button)
                    .child(cross_false_button)
                    .child(TextView::new("single chapter"))
                    .child(Checkbox::new().with_name("single_chapter"))
                    .child(TextView::new("word count"))
                    .child(EditView::new().with_name("word_count"))
                    .child(TextView::new("language"))
                    .child(
                        vec![("".to_string(), None)]
                            .into_iter()
                            .chain(Language::iter().map(|x| (format!("{}", x), Some(x))))
                            .fold(SelectView::new(), |s: SelectView<Option<Language>>, x| {
                                s.item(x.0, x.1)
                            })
                            .with_name("language")
                            .scrollable()
                            .min_height(5),
                    )
                    .child(TextView::new("fandoms"))
                    .child(EditView::new().with_name("fandoms"))
                    .child(TextView::new("rating"))
                    .child(
                        vec![("".to_string(), None)]
                            .into_iter()
                            .chain(Rating::iter().map(|x| (format!("{}", x), Some(x))))
                            .fold(SelectView::new(), |s, x| s.item(x.0, x.1))
                            .with_name("rating")
                            .scrollable()
                            .min_height(5),
                    )
                    .child(TextView::new("warnings"))
                    .child(Warning::iter().fold(LinearLayout::vertical(), |s, x| {
                        s.child(
                            LinearLayout::horizontal()
                                .child(Checkbox::new().with_name(format!("{x}_warning")))
                                .child(TextView::new(format!("{}", x))),
                        )
                    }))
                    .child(TextView::new("categories"))
                    .child(Category::iter().fold(LinearLayout::vertical(), |s, x| {
                        s.child(
                            LinearLayout::horizontal()
                                .child(Checkbox::new().with_name(format!("{x}_category")))
                                .child(TextView::new(format!("{}", x))),
                        )
                    }))
                    .child(TextView::new("characters"))
                    .child(EditView::new().with_name("characters"))
                    .child(TextView::new("relationships"))
                    .child(EditView::new().with_name("relationships"))
                    .child(TextView::new("tags"))
                    .child(EditView::new().with_name("tags"))
                    .child(TextView::new("hits"))
                    .child(EditView::new().with_name("hits"))
                    .child(TextView::new("kudos"))
                    .child(EditView::new().with_name("kudos"))
                    .child(TextView::new("comments"))
                    .child(EditView::new().with_name("comments"))
                    .child(TextView::new("bookmarks"))
                    .child(EditView::new().with_name("bookmarks"))
                    .child(
                        SortBy::iter()
                            .map(|x| (format!("{}", x), x))
                            .fold(SelectView::new(), |s, x| s.item(x.0, x.1))
                            .with_name("sort_by")
                            .scrollable()
                            .min_height(5),
                    )
                    .child(
                        SortDirection::iter()
                            .map(|x| (format!("{}", x), x))
                            .fold(SelectView::new(), |s, x| s.item(x.0, x.1))
                            .with_name("sort_direction")
                            .scrollable()
                            .min_height(5),
                    )
                    .scrollable(),
            )
            .button("search", move |s| {
                let any_field = s
                    .call_on_name("any_field", |view: &mut EditView| view.get_content())
                    .unwrap();
                let title = s
                    .call_on_name("title", |view: &mut EditView| view.get_content())
                    .unwrap();
                let author = s
                    .call_on_name("author", |view: &mut EditView| view.get_content())
                    .unwrap();
                let date = s
                    .call_on_name("date", |view: &mut EditView| view.get_content())
                    .unwrap();
                let completion = compl_group.selection();
                let crossovers = cross_group.selection();
                let single_chapter = s
                    .call_on_name("single_chapter", |view: &mut Checkbox| view.is_checked())
                    .unwrap();
                let word_count = s
                    .call_on_name("word_count", |view: &mut EditView| view.get_content())
                    .unwrap();
                let language = s
                    .call_on_name("language", |view: &mut SelectView<Option<Language>>| {
                        view.selection()
                    })
                    .unwrap()
                    .unwrap();
                let fandoms = s
                    .call_on_name("fandoms", |view: &mut EditView| view.get_content())
                    .unwrap()
                    .split(",")
                    .map(|s| s.to_string())
                    .collect::<HashSet<String>>();
                let ratings = s
                    .call_on_name("rating", |view: &mut SelectView<Option<Rating>>| {
                        view.selection()
                    })
                    .unwrap()
                    .unwrap();
                let warnings: HashSet<Warning> = Warning::iter()
                    .filter(|x| {
                        s.call_on_name(&format!("{x}_warning"), |view: &mut Checkbox| {
                            view.is_checked()
                        })
                        .unwrap()
                    })
                    .collect();
                let categories: Vec<Category> = Category::iter()
                    .filter(|x| {
                        s.call_on_name(&format!("{x}_category"), |view: &mut Checkbox| {
                            view.is_checked()
                        })
                        .unwrap()
                    })
                    .collect();
                let characters = s
                    .call_on_name("characters", |view: &mut EditView| view.get_content())
                    .unwrap();
                let relationships = s
                    .call_on_name("relationships", |view: &mut EditView| view.get_content())
                    .unwrap();
                let tags = s
                    .call_on_name("tags", |view: &mut EditView| view.get_content())
                    .unwrap();
                let hits = s
                    .call_on_name("hits", |view: &mut EditView| view.get_content())
                    .unwrap();
                let kudos = s
                    .call_on_name("kudos", |view: &mut EditView| view.get_content())
                    .unwrap();
                let comments = s
                    .call_on_name("comments", |view: &mut EditView| view.get_content())
                    .unwrap();
                let bookmarks = s
                    .call_on_name("bookmarks", |view: &mut EditView| view.get_content())
                    .unwrap();
                let sort_by = s
                    .call_on_name("sort_by", |view: &mut SelectView<SortBy>| view.selection())
                    .unwrap();
                let sort_direction = s
                    .call_on_name("sort_direction", |view: &mut SelectView<SortDirection>| {
                        view.selection()
                    })
                    .unwrap();

                fn parse_range(s: String) -> Option<Range<usize>> {
                    let split = s.split_once('-')?;

                    Some(split.0.parse::<usize>().ok()?..split.0.parse::<usize>().ok()?)
                }

                SearchQuery::builder()
                    .any(any_field.to_string())
                    .title(title.to_string())
                    .author(author.to_string())
                    .date(date.to_string())
                    .completed(*completion)
                    .crossover(*crossovers)
                    .single_chapter(single_chapter)
                    .word_count(parse_range(word_count.to_string()))
                    .language(Rc::<Option<Language>>::unwrap_or_clone(language))
                    .fandoms(fandoms)
                    .rating(Rc::<Option<Rating>>::unwrap_or_clone(ratings))
                    .warnings(warnings)
                    ;
            }),
    );
    // siv.add_layer(render_work(&work, 0));

    siv.add_global_callback(Event::Key(Key::Right), |s| {
        if let Some(Some(app)) = s.call_on_name("app", |app: &mut App| {
            if app.chapter != app.work.chapters.len() {
                app.chapter += 1;
                Some(app.clone())
            } else {
                None
            }
        }) {
            s.pop_layer();
            s.add_layer(render_work(&app.work, app.chapter));
        }
    });

    siv.add_global_callback(Event::Key(Key::Left), |s| {
        if let Some(Some(app)) = s.call_on_name("app", |app: &mut App| {
            if app.chapter != 0 {
                app.chapter -= 1;
                Some(app.clone())
            } else {
                None
            }
        }) {
            s.pop_layer();
            s.add_layer(render_work(&app.work, app.chapter));
        }
    });

    siv.add_global_callback('q', |s| s.quit());

    // Starts the event loop.
    siv.run();

    // setup terminal
    // enable_raw_mode()?;
    // let mut stdout = io::stdout();
    // execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    // let backend = CrosstermBackend::new(stdout);
    // let mut terminal = Terminal::new(backend)?;

    // // create app and run it
    // let app = App::new();
    // let res = run_app(&mut terminal, app);

    // // restore terminal
    // disable_raw_mode()?;
    // execute!(
    //     terminal.backend_mut(),
    //     LeaveAlternateScreen,
    //     DisableMouseCapture
    // )?;
    // terminal.show_cursor()?;

    // if let Err(err) = res {
    //     println!("{:?}", err)
    // }

    Ok(())
}
