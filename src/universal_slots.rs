use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotConfig {
    pub id: i32,
    pub name: String,
    pub rows: usize,
    pub reels: usize,
    pub is_megaway: bool,
    pub min_megaway_rows: usize,
    pub max_megaway_rows: usize,
    pub default_bet: u32,
    pub min_bet: u32,
    pub max_bet: u32,
    pub wild_enabled: bool,
    pub free_spins_enabled: bool,
    pub rtp_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SymbolType {
    Normal,
    Wild,
    Scatter,
    Bonus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotSymbol {
    pub id: i32,
    pub name: String,
    pub symbol_type: SymbolType,
    pub value: u32,
    pub image_url: Option<String>,
    pub payouts: HashMap<usize, u32>, // 连线数量 -> 赔付倍数
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReelStrip {
    pub reel_number: usize,
    pub symbols: Vec<(SlotSymbol, u32)>, // (符号, 权重)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payline {
    pub line_number: usize,
    pub pattern: Vec<(usize, usize)>, // (卷轴索引, 行索引)
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalSlotMachine {
    pub config: SlotConfig,
    pub symbols: Vec<SlotSymbol>,
    pub reel_strips: Vec<ReelStrip>,
    pub paylines: Vec<Payline>,
}

impl UniversalSlotMachine {
    pub fn new(
        config: SlotConfig,
        symbols: Vec<SlotSymbol>,
        reel_strips: Vec<ReelStrip>,
        paylines: Vec<Payline>,
    ) -> Self {
        Self {
            config,
            symbols,
            reel_strips,
            paylines,
        }
    }

    pub fn spin(&self, bet_per_line: u32) -> UniversalSpinResult {
        let mut rng = rand::rng();
        let mut grid = Vec::new();
        let mut megaway_rows = Vec::new();

        // 生成每个卷轴的结果
        for reel_strip in &self.reel_strips {
            let reel_symbols = if self.config.is_megaway {
                // Megaway模式：每个卷轴的行数随机
                let rows =
                    rng.random_range(self.config.min_megaway_rows..=self.config.max_megaway_rows);
                megaway_rows.push(rows);
                self.spin_reel(reel_strip, rows)
            } else {
                // 标准模式：固定行数
                self.spin_reel(reel_strip, self.config.rows)
            };
            grid.push(reel_symbols);
        }

        // 计算获胜线路
        let winning_lines = if self.config.is_megaway {
            self.check_megaway_wins(&grid)
        } else {
            self.check_standard_wins(&grid)
        };

        // 计算总赢金
        let total_win = self.calculate_total_win(&winning_lines, bet_per_line);

        // 检查免费旋转触发
        let free_spins = if self.config.free_spins_enabled {
            self.check_free_spins(&grid)
        } else {
            0
        };

        UniversalSpinResult {
            grid,
            megaway_rows: if self.config.is_megaway {
                Some(megaway_rows)
            } else {
                None
            },
            winning_lines,
            total_win,
            free_spins,
            bet_per_line,
        }
    }

    fn spin_reel(&self, reel_strip: &ReelStrip, rows: usize) -> Vec<SlotSymbol> {
        let mut rng = rand::rng();
        let mut symbols = Vec::new();

        // 创建加权符号池
        let mut weighted_symbols = Vec::new();
        for (symbol, weight) in &reel_strip.symbols {
            for _ in 0..*weight {
                weighted_symbols.push(symbol.clone());
            }
        }

        // 随机选择符号
        for _ in 0..rows {
            let idx = rng.random_range(0..weighted_symbols.len());
            symbols.push(weighted_symbols[idx].clone());
        }

        symbols
    }

    fn check_standard_wins(&self, grid: &Vec<Vec<SlotSymbol>>) -> Vec<WinningLine> {
        let mut winning_lines = Vec::new();

        for payline in &self.paylines {
            if !payline.is_active {
                continue;
            }

            let mut line_symbols = Vec::new();
            for &(reel_idx, row_idx) in &payline.pattern {
                if reel_idx < grid.len() && row_idx < grid[reel_idx].len() {
                    line_symbols.push(grid[reel_idx][row_idx].clone());
                }
            }

            if let Some(win) = self.check_line_win(&line_symbols) {
                winning_lines.push(WinningLine {
                    payline_number: payline.line_number,
                    symbols: line_symbols,
                    win_multiplier: win,
                });
            }
        }

        winning_lines
    }

    fn check_megaway_wins(&self, grid: &Vec<Vec<SlotSymbol>>) -> Vec<WinningLine> {
        let mut winning_lines = Vec::new();

        // Megaway使用从左到右的连续符号计算
        // 计算所有可能的路径
        let paths = self.calculate_megaway_paths(grid);

        for (path_idx, path) in paths.iter().enumerate() {
            let mut line_symbols = Vec::new();
            for &(reel_idx, row_idx) in path {
                line_symbols.push(grid[reel_idx][row_idx].clone());
            }

            if let Some(win) = self.check_line_win(&line_symbols) {
                winning_lines.push(WinningLine {
                    payline_number: path_idx,
                    symbols: line_symbols,
                    win_multiplier: win,
                });
            }
        }

        winning_lines
    }

    fn calculate_megaway_paths(&self, grid: &Vec<Vec<SlotSymbol>>) -> Vec<Vec<(usize, usize)>> {
        let mut paths = Vec::new();

        // 从第一个卷轴的每个符号开始
        for start_row in 0..grid[0].len() {
            self.find_paths_recursive(grid, 0, start_row, vec![(0, start_row)], &mut paths);
        }

        paths
    }

    fn find_paths_recursive(
        &self,
        grid: &Vec<Vec<SlotSymbol>>,
        reel_idx: usize,
        _row_idx: usize,
        current_path: Vec<(usize, usize)>,
        paths: &mut Vec<Vec<(usize, usize)>>,
    ) {
        // 如果到达最后一个卷轴，保存路径
        if reel_idx == grid.len() - 1 {
            paths.push(current_path);
            return;
        }

        // 连接到下一个卷轴的所有符号
        let next_reel_idx = reel_idx + 1;
        if next_reel_idx < grid.len() {
            for next_row in 0..grid[next_reel_idx].len() {
                let mut new_path = current_path.clone();
                new_path.push((next_reel_idx, next_row));
                self.find_paths_recursive(grid, next_reel_idx, next_row, new_path, paths);
            }
        }
    }

    fn check_line_win(&self, symbols: &[SlotSymbol]) -> Option<u32> {
        if symbols.is_empty() {
            return None;
        }

        let first_symbol = &symbols[0];
        let mut consecutive_count = 1;
        let mut has_wild = false;

        // 检查从左到右的连续符号
        for i in 1..symbols.len() {
            let current = &symbols[i];

            if current.symbol_type == SymbolType::Wild && self.config.wild_enabled {
                has_wild = true;
                consecutive_count += 1;
            } else if current.name == first_symbol.name
                || (first_symbol.symbol_type == SymbolType::Wild && self.config.wild_enabled)
            {
                consecutive_count += 1;
            } else {
                break;
            }
        }

        // 至少需要3个连续符号才算赢
        if consecutive_count >= 3 {
            // 获取赔付倍数
            let base_symbol = if first_symbol.symbol_type == SymbolType::Wild {
                // 如果第一个是Wild，找第一个非Wild符号
                symbols.iter().find(|s| s.symbol_type != SymbolType::Wild)?
            } else {
                first_symbol
            };

            if let Some(&payout) = base_symbol.payouts.get(&consecutive_count) {
                // 如果有Wild，可能增加赔付
                let multiplier = if has_wild && !self.config.is_megaway {
                    2
                } else {
                    1
                };
                return Some(payout * multiplier);
            }
        }

        None
    }

    fn check_free_spins(&self, grid: &Vec<Vec<SlotSymbol>>) -> u32 {
        let mut scatter_count = 0;

        // 计算Scatter符号数量
        for reel in grid {
            for symbol in reel {
                if symbol.symbol_type == SymbolType::Scatter {
                    scatter_count += 1;
                }
            }
        }

        // 通常3个或更多Scatter触发免费旋转
        match scatter_count {
            3 => 10,
            4 => 15,
            5 => 20,
            6.. => 25,
            _ => 0,
        }
    }

    fn calculate_total_win(&self, winning_lines: &[WinningLine], bet_per_line: u32) -> u32 {
        let mut total = 0;

        if self.config.is_megaway {
            // Megaway模式：所有赢线的赢金相加
            for line in winning_lines {
                total += line.win_multiplier * bet_per_line;
            }
        } else {
            // 标准模式：每条支付线独立计算
            for line in winning_lines {
                total += line.win_multiplier * bet_per_line;
            }
        }

        total
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalSpinResult {
    pub grid: Vec<Vec<SlotSymbol>>,
    pub megaway_rows: Option<Vec<usize>>,
    pub winning_lines: Vec<WinningLine>,
    pub total_win: u32,
    pub free_spins: u32,
    pub bet_per_line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinningLine {
    pub payline_number: usize,
    pub symbols: Vec<SlotSymbol>,
    pub win_multiplier: u32,
}

// 用于从数据库构建SlotMachine的辅助结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotConfigBuilder {
    pub config: SlotConfig,
    pub symbols: Vec<SlotSymbol>,
    pub reel_compositions: HashMap<usize, Vec<(i32, u32)>>, // reel_number -> [(symbol_id, weight)]
    pub paylines: Vec<Payline>,
}

impl SlotConfigBuilder {
    pub fn build(self) -> UniversalSlotMachine {
        let mut reel_strips = Vec::new();

        // 构建卷轴条
        for i in 0..self.config.reels {
            if let Some(compositions) = self.reel_compositions.get(&i) {
                let mut symbols_with_weights = Vec::new();

                for (symbol_id, weight) in compositions {
                    if let Some(symbol) = self.symbols.iter().find(|s| s.id == *symbol_id) {
                        symbols_with_weights.push((symbol.clone(), *weight));
                    }
                }

                reel_strips.push(ReelStrip {
                    reel_number: i,
                    symbols: symbols_with_weights,
                });
            }
        }

        UniversalSlotMachine::new(self.config, self.symbols, reel_strips, self.paylines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_slot_creation() {
        let config = SlotConfig {
            id: 1,
            name: "Test Slot".to_string(),
            rows: 3,
            reels: 5,
            is_megaway: false,
            min_megaway_rows: 2,
            max_megaway_rows: 7,
            default_bet: 1,
            min_bet: 1,
            max_bet: 100,
            wild_enabled: true,
            free_spins_enabled: true,
            rtp_percentage: 96.0,
        };

        let mut payouts = HashMap::new();
        payouts.insert(3, 10);
        payouts.insert(4, 20);
        payouts.insert(5, 50);

        let symbol = SlotSymbol {
            id: 1,
            name: "Cherry".to_string(),
            symbol_type: SymbolType::Normal,
            value: 10,
            image_url: None,
            payouts,
        };

        let reel_strip = ReelStrip {
            reel_number: 0,
            symbols: vec![(symbol.clone(), 10)],
        };

        let payline = Payline {
            line_number: 1,
            pattern: vec![(0, 1), (1, 1), (2, 1), (3, 1), (4, 1)],
            is_active: true,
        };

        let machine =
            UniversalSlotMachine::new(config, vec![symbol], vec![reel_strip; 5], vec![payline]);

        assert_eq!(machine.config.reels, 5);
        assert_eq!(machine.config.rows, 3);
    }
}
