use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Symbol {
    Cherry,
    Lemon,
    Orange,
    Plum,
    Bell,
    Bar,
    Seven,
    Diamond,
}

impl Symbol {
    #[allow(dead_code)]
    fn value(&self) -> u32 {
        match self {
            Symbol::Cherry => 2,
            Symbol::Lemon => 3,
            Symbol::Orange => 5,
            Symbol::Plum => 8,
            Symbol::Bell => 10,
            Symbol::Bar => 15,
            Symbol::Seven => 25,
            Symbol::Diamond => 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reel {
    symbols: Vec<Symbol>,
    position: usize,
}

impl Reel {
    pub fn new() -> Self {
        // Standard slot machine reel configuration with more variety to reduce wins
        let symbols = vec![
            Symbol::Cherry, Symbol::Lemon, Symbol::Orange, Symbol::Plum,
            Symbol::Bell, Symbol::Bar, Symbol::Seven, Symbol::Diamond,
            Symbol::Lemon, Symbol::Orange, Symbol::Plum, Symbol::Cherry,
            Symbol::Bell, Symbol::Lemon, Symbol::Orange, Symbol::Bar,
            Symbol::Plum, Symbol::Cherry, Symbol::Bell, Symbol::Lemon,
            Symbol::Orange, Symbol::Plum, Symbol::Cherry, Symbol::Bell,
            Symbol::Bar, Symbol::Seven, Symbol::Diamond, Symbol::Lemon,
        ];
        
        Self {
            symbols,
            position: 0,
        }
    }
    
    pub fn spin(&mut self) -> Symbol {
        let mut rng = rand::rng();
        self.position = rng.random_range(0..self.symbols.len());
        self.symbols[self.position].clone()
    }
    
    #[allow(dead_code)]
    pub fn get_visible_symbols(&self, count: usize) -> Vec<Symbol> {
        let mut result = Vec::new();
        for i in 0..count {
            let pos = (self.position + i) % self.symbols.len();
            result.push(self.symbols[pos].clone());
        }
        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotMachine {
    pub reels: Vec<Reel>,
    pub rows: usize,
    pub bet_multipliers: HashMap<String, u32>,
}

impl SlotMachine {
    pub fn new(reel_count: usize, rows: usize) -> Self {
        let mut bet_multipliers = HashMap::new();
        
        // Define winning combinations and their multipliers (adjusted for proper RTP)
        bet_multipliers.insert("three_of_kind".to_string(), 6);
        bet_multipliers.insert("three_sevens".to_string(), 45);
        bet_multipliers.insert("three_diamonds".to_string(), 90);
        bet_multipliers.insert("mixed_bars".to_string(), 5);
        
        Self {
            reels: (0..reel_count).map(|_| Reel::new()).collect(),
            rows,
            bet_multipliers,
        }
    }
    
    pub fn spin(&mut self) -> SpinResult {
        let mut grid = Vec::new();
        
        for reel in &mut self.reels {
            let symbols = (0..self.rows).map(|_| reel.spin()).collect();
            grid.push(symbols);
        }
        
        let winning_lines = self.check_winning_lines(&grid);
        let total_win = self.calculate_winnings(&winning_lines);
        
        SpinResult {
            grid,
            winning_lines,
            total_win,
        }
    }
    
    fn check_winning_lines(&self, grid: &Vec<Vec<Symbol>>) -> Vec<WinningLine> {
        let mut winning_lines = Vec::new();
        
        // Check horizontal lines
        for row in 0..self.rows {
            let mut line_symbols = Vec::new();
            for col in 0..self.reels.len() {
                line_symbols.push(grid[col][row].clone());
            }
            
            if let Some(win) = self.check_line(&line_symbols) {
                winning_lines.push(WinningLine {
                    line_type: LineType::Horizontal(row),
                    symbols: line_symbols,
                    win_type: win,
                });
            }
        }
        
        // Check diagonal lines (if 3x3 or larger)
        if self.reels.len() >= 3 && self.rows >= 3 {
            // Top-left to bottom-right
            let mut diagonal1 = Vec::new();
            for i in 0..self.reels.len().min(self.rows) {
                diagonal1.push(grid[i][i].clone());
            }
            if let Some(win) = self.check_line(&diagonal1) {
                winning_lines.push(WinningLine {
                    line_type: LineType::DiagonalDown,
                    symbols: diagonal1,
                    win_type: win,
                });
            }
            
            // Top-right to bottom-left
            let mut diagonal2 = Vec::new();
            for i in 0..self.reels.len().min(self.rows) {
                diagonal2.push(grid[i][self.rows - 1 - i].clone());
            }
            if let Some(win) = self.check_line(&diagonal2) {
                winning_lines.push(WinningLine {
                    line_type: LineType::DiagonalUp,
                    symbols: diagonal2,
                    win_type: win,
                });
            }
        }
        
        winning_lines
    }
    
    fn check_line(&self, symbols: &[Symbol]) -> Option<WinType> {
        if symbols.len() < 3 {
            return None;
        }
        
        // Only check for three-of-a-kind wins (all three symbols must match)
        let first = &symbols[0];
        let second = &symbols[1];
        let third = &symbols[2];
        
        if std::mem::discriminant(first) == std::mem::discriminant(second) && 
           std::mem::discriminant(second) == std::mem::discriminant(third) {
            return match first {
                Symbol::Seven => Some(WinType::ThreeSevens),
                Symbol::Diamond => Some(WinType::ThreeDiamonds),
                Symbol::Bar => Some(WinType::MixedBars),
                _ => Some(WinType::ThreeOfKind(first.clone())),
            };
        }
        
        None
    }
    
    fn calculate_winnings(&self, winning_lines: &[WinningLine]) -> u32 {
        let mut total = 0;
        
        for line in winning_lines {
            let multiplier = match &line.win_type {
                WinType::ThreeSevens => self.bet_multipliers.get("three_sevens").unwrap_or(&45),
                WinType::ThreeDiamonds => self.bet_multipliers.get("three_diamonds").unwrap_or(&90),
                WinType::ThreeOfKind(_symbol) => {
                    let base = self.bet_multipliers.get("three_of_kind").unwrap_or(&6);
                    base
                },
                WinType::MixedBars => self.bet_multipliers.get("mixed_bars").unwrap_or(&5),
            };
            
            total += multiplier;
        }
        
        total
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpinResult {
    pub grid: Vec<Vec<Symbol>>,
    pub winning_lines: Vec<WinningLine>,
    pub total_win: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinningLine {
    pub line_type: LineType,
    pub symbols: Vec<Symbol>,
    pub win_type: WinType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineType {
    Horizontal(usize),
    DiagonalDown,
    DiagonalUp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WinType {
    ThreeSevens,
    ThreeDiamonds,
    ThreeOfKind(Symbol),
    MixedBars,
}

// RTP (Return to Player) calculation
pub fn calculate_rtp(spins: u32) -> f64 {
    let mut machine = SlotMachine::new(3, 3);
    let mut total_bet = 0u64;
    let mut total_win = 0u64;
    
    for _ in 0..spins {
        total_bet += 1; // Assuming 1 credit per spin
        let result = machine.spin();
        total_win += result.total_win as u64;
    }
    
    (total_win as f64 / total_bet as f64) * 100.0
}

// Progressive jackpot implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressiveJackpot {
    pub current_amount: u64,
    pub contribution_rate: f64, // Percentage of each bet that goes to jackpot
    pub min_amount: u64,
    pub last_won: Option<chrono::DateTime<chrono::Utc>>,
}

impl ProgressiveJackpot {
    pub fn new(min_amount: u64, contribution_rate: f64) -> Self {
        Self {
            current_amount: min_amount,
            contribution_rate,
            min_amount,
            last_won: None,
        }
    }
    
    pub fn add_contribution(&mut self, bet_amount: u64) {
        let contribution = (bet_amount as f64 * self.contribution_rate) as u64;
        self.current_amount += contribution;
    }
    
    pub fn check_and_award(&mut self, is_jackpot_win: bool) -> Option<u64> {
        if is_jackpot_win {
            let win_amount = self.current_amount;
            self.current_amount = self.min_amount;
            self.last_won = Some(chrono::Utc::now());
            Some(win_amount)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_slot_machine_creation() {
        let machine = SlotMachine::new(3, 3);
        assert_eq!(machine.reels.len(), 3);
        assert_eq!(machine.rows, 3);
    }
    
    #[test]
    fn test_rtp_calculation() {
        let rtp = calculate_rtp(100000);
        println!("RTP over 100000 spins: {:.2}%", rtp);
        // RTP should be between 75% and 105% for a fair slot machine
        // Allow variance for randomness in testing
        assert!(rtp > 75.0 && rtp < 105.0);
    }
    
    #[test]
    fn test_progressive_jackpot() {
        let mut jackpot = ProgressiveJackpot::new(1000, 0.02);
        jackpot.add_contribution(100);
        assert_eq!(jackpot.current_amount, 1002);
        
        let win = jackpot.check_and_award(true);
        assert_eq!(win, Some(1002));
        assert_eq!(jackpot.current_amount, 1000);
    }
}