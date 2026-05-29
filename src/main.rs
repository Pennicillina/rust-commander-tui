use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};

#[derive(Clone)]
struct ElementoFile {
    nome: String,
    percorso: PathBuf,
    is_directory: bool,
    dimensione: u64,
}

struct Pannello {
    percorso_corrente: PathBuf,
    elementi: Vec<ElementoFile>,
    indice_selezionato: usize,
}

impl Pannello {
    fn new(percorso_iniziale: PathBuf) -> Self {
        let mut pannello = Self {
            percorso_corrente: percorso_iniziale.canonicalize().unwrap_or(percorso_iniziale),
            elementi: Vec::new(),
            indice_selezionato: 0,
        };
        pannello.aggiorna_lista();
        pannello
    }

    fn aggiorna_lista(&mut self) {
        let vecchio_selezionato = self.indice_selezionato;
        self.elementi.clear();
        
        // cartella superiore
        if let Some(genitore) = self.percorso_corrente.parent() {
            self.elementi.push(ElementoFile {
                nome: String::from(".. (Cartella Superiore)"),
                percorso: genitore.to_path_buf(),
                is_directory: true,
                dimensione: 0,
            });
        }

        // directory corrente
        if let Ok(voci) = fs::read_dir(&self.percorso_corrente) {
            for voce in voci {
                if let Ok(entry) = voce {
                    let nome = entry.file_name().to_string_lossy().into_owned();
                    let percorso = entry.path();
                    let mut is_directory = false;
                    let mut dimensione = 0;
                    if let Ok(metadata) = entry.metadata() {
                        is_directory = metadata.is_dir();
                        dimensione = metadata.len();
                    }
                    self.elementi.push(ElementoFile { nome, percorso, is_directory, dimensione });
                }
            }
        }

        // prima le cartelle, poi i file
        self.elementi.sort_by(|a, b| {
            if a.is_directory != b.is_directory {
                b.is_directory.cmp(&a.is_directory)
            } else {
                a.nome.to_lowercase().cmp(&b.nome.to_lowercase())
            }
        });

        if vecchio_selezionato >= self.elementi.len() {
            self.indice_selezionato = if !self.elementi.is_empty() { self.elementi.len() - 1 } else { 0 };
        } else {
            self.indice_selezionato = vecchio_selezionato;
        }
    }

    fn muovi_su(&mut self) {
        if self.indice_selezionato > 0 {
            self.indice_selezionato -= 1;
        }
    }

    fn muovi_giu(&mut self) {
        if !self.elementi.is_empty() && self.indice_selezionato < self.elementi.len() - 1 {
            self.indice_selezionato += 1;
        }
    }
}

struct App {
    pannello_sinistro: Pannello,
    pannello_destro: Pannello,
    sinistro_attivo: bool,
    messaggio_stato: String,
}

impl App {
    fn pannello_attivo(&mut self) -> &mut Pannello {
        if self.sinistro_attivo {
            &mut self.pannello_sinistro
        } else {
            &mut self.pannello_destro
        }
    }

    fn pannello_destinazione(&mut self) -> &mut Pannello {
        if self.sinistro_attivo {
            &mut self.pannello_destro
        } else {
            &mut self.pannello_sinistro
        }
    }

    fn copia_cartella_ricorsiva(sorgente: &Path, destinazione: &Path) -> io::Result<()> {
        fs::create_dir_all(destinazione)?;
        for voce in fs::read_dir(sorgente)? {
            let voce = voce?;
            let tipo_file = voce.file_type()?;
            let nome_file = voce.file_name();
            if tipo_file.is_dir() {
                Self::copia_cartella_ricorsiva(&voce.path(), &destinazione.join(nome_file))?;
            } else {
                fs::copy(voce.path(), destinazione.join(nome_file))?;
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // cartella del progetto ".")
    let mut app = App {
        pannello_sinistro: Pannello::new(PathBuf::from(".")),
        pannello_destro: Pannello::new(PathBuf::from(".")),
        sinistro_attivo: true,
        messaggio_stato: String::from(" F5: Copia | F8: Elimina | Tab/Click Mouse: Cambia Pannello | Backspace: Indietro | q: Esci"),
    };

    loop {
        terminal.draw(|f| {
            // barra comandi (3 righe)
            let layout_verticale = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(f.size());

            let layout_orizzontale = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(layout_verticale[0]);

            let stile_selezione = Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD);

            let larghezza_pannello = (layout_orizzontale[0].width as usize).saturating_sub(5);

            let mut elementi_sinistri = Vec::new();
            for item in &app.pannello_sinistro.elementi {
                let prefisso = if item.is_directory { "[DIR] " } else { "[FIL] " };
                let testo_sinistro = format!("{}{}", prefisso, item.nome);
                
                let testo_destro = if item.is_directory {
                    if item.nome.contains("..") { String::new() } else { String::from("<DIR>") }
                } else {
                    format!("{:.1} KB", (item.dimensione as f64) / 1024.0)
                };

                let spazio_disponibile = larghezza_pannello.saturating_sub(testo_sinistro.chars().count());
                let spazi = " ".repeat(spazio_disponibile.saturating_sub(testo_destro.chars().count()));
                let riga_completa = format!("{}{}{}", testo_sinistro, spazi, testo_destro);
                
                elementi_sinistri.push(ListItem::new(riga_completa));
            }
            
            let mut stato_sinistro = ListState::default();
            stato_sinistro.select(Some(app.pannello_sinistro.indice_selezionato));
            let colore_bordo_sinistro = if app.sinistro_attivo { Color::Green } else { Color::DarkGray };
            let titolo_sinistro = format!(" Sinistro: {} ", app.pannello_sinistro.percorso_corrente.display());
            let lista_sinistra = List::new(elementi_sinistri)
                .block(Block::default().title(titolo_sinistro).borders(Borders::ALL).border_style(Style::default().fg(colore_bordo_sinistro)))
                .highlight_style(stile_selezione)
                .highlight_symbol(">> ");
            f.render_stateful_widget(lista_sinistra, layout_orizzontale[0], &mut stato_sinistro);

            let mut elementi_destri = Vec::new();
            for item in &app.pannello_destro.elementi {
                let prefisso = if item.is_directory { "[DIR] " } else { "[FIL] " };
                let testo_sinistro = format!("{}{}", prefisso, item.nome);
                
                let testo_destro = if item.is_directory {
                    if item.nome.contains("..") { String::new() } else { String::from("<DIR>") }
                } else {
                    format!("{:.1} KB", (item.dimensione as f64) / 1024.0)
                };

                let spazio_disponibile = larghezza_pannello.saturating_sub(testo_sinistro.chars().count());
                let spazi = " ".repeat(spazio_disponibile.saturating_sub(testo_destro.chars().count()));
                let riga_completa = format!("{}{}{}", testo_sinistro, spazi, testo_destro);
                
                elementi_destri.push(ListItem::new(riga_completa));
            }
            
            let mut stato_destro = ListState::default();
            stato_destro.select(Some(app.pannello_destro.indice_selezionato));
            let colore_bordo_destro = if !app.sinistro_attivo { Color::Green } else { Color::DarkGray };
            let titolo_destro = format!(" Destro: {} ", app.pannello_destro.percorso_corrente.display());
            let lista_destra = List::new(elementi_destri)
                .block(Block::default().title(titolo_destro).borders(Borders::ALL).border_style(Style::default().fg(colore_bordo_destro)))
                .highlight_style(stile_selezione)
                .highlight_symbol(">> ");
            f.render_stateful_widget(lista_destra, layout_orizzontale[1], &mut stato_destro);

            let barra_stato = Paragraph::new(app.messaggio_stato.as_str())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title(" Comandi Rapidi "));
            f.render_widget(barra_stato, layout_verticale[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                // TASTIERA
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => break,
                            
                            KeyCode::Up => { app.pannello_attivo().muovi_su(); }
                            KeyCode::Down => { app.pannello_attivo().muovi_giu(); }
                            KeyCode::Tab => { app.sinistro_attivo = !app.sinistro_attivo; }

                            KeyCode::Enter => {
                                let pannello = app.pannello_attivo();
                                if !pannello.elementi.is_empty() {
                                    let elemento = &pannello.elementi[pannello.indice_selezionato];
                                    if elemento.is_directory {
                                        pannello.percorso_corrente = elemento.percorso.clone();
                                        pannello.aggiorna_lista();
                                    }
                                }
                            }

                            KeyCode::Backspace => {
                                let pannello = app.pannello_attivo();
                                if let Some(genitore) = pannello.percorso_corrente.parent() {
                                    pannello.percorso_corrente = genitore.to_path_buf();
                                    pannello.aggiorna_lista();
                                }
                            }

                            // Tasto F5
                            KeyCode::F(5) => {
                                let (sorgente_path, nome_file, is_dir) = {
                                    let src = app.pannello_attivo();
                                    if src.elementi.is_empty() || (src.indice_selezionato == 0 && src.elementi[0].nome.contains("..")) {
                                        (None, String::new(), false)
                                    } else {
                                        let elem = &src.elementi[src.indice_selezionato];
                                        (Some(elem.percorso.clone()), elem.nome.clone(), elem.is_directory)
                                    }
                                };

                                if let Some(src_path) = sorgente_path {
                                    let dest_dir = app.pannello_destinazione().percorso_corrente.clone();
                                    let dest_path = dest_dir.join(nome_file);

                                    let risultato = if is_dir {
                                        App::copia_cartella_ricorsiva(&src_path, &dest_path)
                                    } else {
                                        fs::copy(&src_path, &dest_path).map(|_| ())
                                    };

                                    match risultato {
                                        Ok(_) => app.messaggio_stato = String::from("Copia completata con successo!"),
                                        Err(e) => app.messaggio_stato = format!("Errore di copia: {}", e),
                                    }

                                    app.pannello_sinistro.aggiorna_lista();
                                    app.pannello_destro.aggiorna_lista();
                                } else {
                                    app.messaggio_stato = String::from("Impossibile copiare questo elemento.");
                                }
                            }

                            // Tasto F8
                            KeyCode::F(8) => {
                                let sorgente_da_eliminare = {
                                    let src = app.pannello_attivo();
                                    if src.elementi.is_empty() || (src.indice_selezionato == 0 && src.elementi[0].nome.contains("..")) {
                                        None
                                    } else {
                                        Some(src.elementi[src.indice_selezionato].clone())
                                    }
                                };

                                if let Some(elem) = sorgente_da_eliminare {
                                    let risultato = if elem.is_directory {
                                        fs::remove_dir_all(&elem.percorso)
                                    } else {
                                        fs::remove_file(&elem.percorso)
                                    };

                                    match risultato {
                                        Ok(_) => app.messaggio_stato = format!("Eliminato con successo: {}", elem.nome),
                                        Err(e) => app.messaggio_stato = format!("Errore eliminazione: {}", e),
                                    }

                                    app.pannello_sinistro.aggiorna_lista();
                                    app.pannello_destro.aggiorna_lista();
                                } else {
                                    app.messaggio_stato = String::from("Impossibile eliminare la cartella superiore.");
                                }
                            }

                            _ => {}
                        }
                    }
                }

                // EVENTI MOUSE
                Event::Mouse(mouse_event) => {
                    if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                        let colonna_cliccata = mouse_event.column;
                        let riga_cliccata = mouse_event.row as usize;
                        let meta_schermo = (terminal.size()?.width / 2) as u16;

                        if colonna_cliccata < meta_schermo {
                            app.sinistro_attivo = true;
                            if riga_cliccata > 0 && riga_cliccata <= app.pannello_sinistro.elementi.len() {
                                app.pannello_sinistro.indice_selezionato = riga_cliccata - 1;
                            }
                        } else {
                            app.sinistro_attivo = false;
                            if riga_cliccata > 0 && riga_cliccata <= app.pannello_destro.elementi.len() {
                                app.pannello_destro.indice_selezionato = riga_cliccata - 1;
                            }
                        }
                    }
                }

                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}
