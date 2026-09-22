use crossterm::event::{KeyCode, KeyEvent};
use zeroize::Zeroizing;

use crate::vault::{self, Entry, Vault};

#[derive(PartialEq, Eq)]
pub enum AppState {
    Unlock,
    List,
    Detail,
    ConfirmDelete,
    AddForm,
    Quit,
}

pub struct App {
    pub state: AppState,
    pub vault_path: String,

    // Unlock screen
    pub unlock_input: String,
    pub unlock_error: Option<String>,

    // Loaded vault — Some once unlocked
    pub vault: Option<Vault>,
    pub master: Option<Zeroizing<String>>,

    // List screen
    pub selected: usize,

    pub form: AddForm,   

    // Transient status line
    pub status: Option<String>,
}

impl App {
    pub fn new(vault_path: String) -> Self {
        Self {
            state: AppState::Unlock,
            vault_path,
            unlock_input: String::new(),
            unlock_error: None,
            vault: None,
            master: None,
            selected: 0,
            form: AddForm::new(),
            status: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.state {
            AppState::Unlock => self.handle_unlock(key),
            AppState::List => self.handle_list(key),
            AppState::Detail => self.handle_detail(key),
            AppState::ConfirmDelete => self.handle_confirm_delete(key),
            AppState::AddForm => self.handle_add_form(key),
            AppState::Quit => {}
        }
    }

    fn handle_unlock(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.unlock_input.push(c);
                self.unlock_error = None;
            }
            KeyCode::Backspace => {
                self.unlock_input.pop();
                self.unlock_error = None;
            }
            KeyCode::Enter => self.try_unlock(),
            KeyCode::Esc => self.state = AppState::Quit,
            _ => {}
        }
    }

    fn try_unlock(&mut self) {
        match vault::load_vault(&self.vault_path, &self.unlock_input) {
            Ok(v) => {
                self.master = Some(Zeroizing::new(std::mem::take(&mut self.unlock_input)));
                self.vault = Some(v);
                self.selected = 0;
                self.state = AppState::List;
            }
            Err(e) if e.is_wrong_password() => {
                self.unlock_error = Some("Wrong password".into());
                self.unlock_input.clear();
            }
            Err(e) => {
                self.unlock_error = Some(format!("{}", e));
                self.unlock_input.clear();
            }
        }
    }

    fn handle_list(&mut self, key: KeyEvent) {
        let len = self.vault.as_ref().map(|v| v.entries.len()).unwrap_or(0);
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.state = AppState::Quit,
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < len {
                    self.selected += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                if len > 0 {
                    self.state = AppState::Detail;
                }
            }
            KeyCode::Char('d') => {
                if self.current_entry().is_some() {
                    self.state = AppState::ConfirmDelete;
                }
            }
            KeyCode::Char('n') => {
                self.form = AddForm::new();
                self.state = AppState::AddForm;
            }
            _ => {}
        }
    }

    fn handle_detail(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.state = AppState::List,
            _ => {}
        }
    }

    fn handle_confirm_delete(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => self.delete_current(),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.state = AppState::List;
            }
            _ => {}
        }
    }

    fn delete_current(&mut self) {
        let Some(entry) = self.current_entry() else {
            self.state = AppState::List;
            return;
        };
        let site = entry.site.clone();

        let Some(vault) = self.vault.as_mut() else { return; };
        let Some(master) = self.master.as_ref() else { return; };

        if let Err(e) = vault.remove(&site) {
            self.status = Some(format!("Delete failed: {}", e));
            self.state = AppState::List;
            return;
        }

        match vault::save_vault(&self.vault_path, vault, master) {
            Ok(()) => {
                self.status = Some(format!("Deleted '{}'", site));
                let len = vault.entries.len();
                if len == 0 {
                    self.selected = 0;
                } else if self.selected >= len {
                    self.selected = len - 1;
                }
            }
            Err(e) => {
                self.status = Some(format!("Save failed: {}", e));
            }
        }
        self.state = AppState::List;
    }

    pub fn current_entry(&self) -> Option<&Entry> {
        self.vault.as_ref()?.entries.get(self.selected)
    }

    fn handle_add_form(&mut self, key: KeyEvent) {
    // Enter on the last field submits; on earlier fields it advances.
    // Esc cancels. Tab/BackTab always cycles.
    match key.code {
        KeyCode::Esc => {
            self.state = AppState::List;
            self.form = AddForm::new();
        }
        KeyCode::Tab | KeyCode::Down => {
            self.form.focus = (self.form.focus + 1) % 4;
        }
        KeyCode::BackTab | KeyCode::Up => {
            self.form.focus = (self.form.focus + 3) % 4;   // -1 mod 4
        }
        KeyCode::Enter => {
            if self.form.focus == 3 {
                self.submit_add_form();
            } else {
                self.form.focus += 1;
            }
        }
        KeyCode::Backspace => {
            self.form.fields[self.form.focus].pop();
            self.form.error = None;
        }
        KeyCode::Char(c) => {
            self.form.fields[self.form.focus].push(c);
            self.form.error = None;
        }
        _ => {}
    }
}

fn submit_add_form(&mut self) {
    let site = self.form.fields[0].clone();
    let user = self.form.fields[1].clone();
    let password = self.form.fields[2].clone();
    let notes = self.form.fields[3].clone();

    if site.is_empty() {
        self.form.error = Some("site is required".into());
        self.form.focus = 0;
        return;
    }
    if password.is_empty() {
        self.form.error = Some("password is required".into());
        self.form.focus = 2;
        return;
    }

    let Some(vault) = self.vault.as_mut() else { return; };
    let Some(master) = self.master.as_ref() else { return; };

    if let Err(e) = vault.add(Entry { site: site.clone(), user, password, notes }) {
        self.form.error = Some(format!("{}", e));
        return;
    }

    match vault::save_vault(&self.vault_path, vault, master) {
        Ok(()) => {
            self.status = Some(format!("Added '{}'", site));
            self.form = AddForm::new();
            self.state = AppState::List;
            // jump selection to the newly added (last) entry
            self.selected = vault.entries.len().saturating_sub(1);
        }
        Err(e) => {
            self.form.error = Some(format!("Save failed: {}", e));
        }
    }
  }    
 
}

pub struct AddForm {
    pub fields: [String; 4],  // site, user, password, notes
    pub focus: usize,         // 0..=3
    pub error: Option<String>,
}

impl AddForm {
    pub fn new() -> Self {
        Self {
            fields: Default::default(),
            focus: 0,
            error: None,
        }
    }
}
