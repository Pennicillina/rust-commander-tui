# Rust Commander TUI 🦀

Un file manager old school (stile Total Commander / Midnight Commander) leggero e reattivo, sviluppato interamente in **Rust** utilizzando le librerie **Ratatui** e **Crossterm**. È progettato per funzionare direttamente all'interno del terminale, offrendo un'interfaccia a due pannelli navigabile sia tramite tastiera che tramite mouse. 

## ✨ Funzionalità

* **Layout a Doppio Pannello**: Visualizzazione simultanea di due directory con colonne reattive che si adattano alle dimensioni del terminale.
* **Calcolo delle Dimensioni**: Mostra in tempo reale la dimensione di ogni file allineata sul bordo destro o il tag `<DIR>` per le cartelle.
* **Operazioni sui File**: Copia file o intere cartelle ricorsivamente con `F5` ed eliminali in sicurezza con `F8`.
* **Input Ibrido**: Supporto completo per le frecce direzionali/tastiera e integrazione nativa con il **mouse** per cambiare pannello o selezionare file con un click.

## ⌨️ Comandi Rapidi

| Tasto | Azione |
| --- | --- |
| `Freccia Su / Giù` | Muove il cursore tra i file |
| `Tab` / `Click Mouse` | Cambia il focus tra il pannello sinistro e destro |
| `Invio` | Entra nella cartella selezionata |
| `Backspace` | Torna alla cartella superiore |
| `F5` | Copia l'elemento selezionato nel pannello opposto |
| `F8` | Elimina l'elemento selezionato dal disco |
| `q` | Esci dal programma |

## 🚀 Requisiti e Installazione

Assicurati di avere installato [Rust e Cargo](https://www.rust-lang.org/).

1. Clona la repository o scarica i file.
2. Apri il terminale nella cartella del progetto.
3. Esegui il software con il comando:
   ```bash
   cargo run
