CREATE TABLE IF NOT EXISTS rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    player_x_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    player_o_id UUID REFERENCES users(id) ON DELETE CASCADE,

    board_state VARCHAR(9) NOT NULL DEFAULT '---------',
    next_turn CHAR(1) NOT NULL DEFAULT 'X',

    winner CHAR(1),
    status TEXT NOT NULL DEFAULT 'waiting',

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_rooms_player_x ON rooms(player_x_id);
CREATE INDEX idx_rooms_player_o ON rooms(player_o_id);
