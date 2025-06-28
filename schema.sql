-- Drop existing tables if they exist
DROP TABLE IF EXISTS slot_spin_history;
DROP TABLE IF EXISTS jackpot_wins;
DROP TABLE IF EXISTS game_sessions;
DROP TABLE IF EXISTS players;
DROP TABLE IF EXISTS todos;

-- Basic todos table (keeping for compatibility)
CREATE TABLE IF NOT EXISTS todos (
  id serial PRIMARY KEY,
  note TEXT NOT NULL
);

-- Players table
CREATE TABLE IF NOT EXISTS players (
  id serial PRIMARY KEY,
  username VARCHAR(50) UNIQUE NOT NULL,
  email VARCHAR(100) UNIQUE NOT NULL,
  balance BIGINT DEFAULT 1000,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  last_login TIMESTAMP WITH TIME ZONE
);

-- Game sessions table
CREATE TABLE IF NOT EXISTS game_sessions (
  id serial PRIMARY KEY,
  player_id INTEGER REFERENCES players(id),
  session_start TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  session_end TIMESTAMP WITH TIME ZONE,
  total_spins INTEGER DEFAULT 0,
  total_bet BIGINT DEFAULT 0,
  total_won BIGINT DEFAULT 0
);

-- Slot spin history
CREATE TABLE IF NOT EXISTS slot_spin_history (
  id serial PRIMARY KEY,
  player_id INTEGER REFERENCES players(id),
  session_id INTEGER REFERENCES game_sessions(id),
  bet_amount BIGINT NOT NULL,
  win_amount BIGINT NOT NULL,
  symbols JSONB NOT NULL,
  winning_lines JSONB,
  spin_time TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Jackpot wins history
CREATE TABLE IF NOT EXISTS jackpot_wins (
  id serial PRIMARY KEY,
  player_id INTEGER REFERENCES players(id),
  win_amount BIGINT NOT NULL,
  win_time TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better performance
CREATE INDEX idx_players_username ON players(username);
CREATE INDEX idx_sessions_player_id ON game_sessions(player_id);
CREATE INDEX idx_spins_player_id ON slot_spin_history(player_id);
CREATE INDEX idx_spins_session_id ON slot_spin_history(session_id);
