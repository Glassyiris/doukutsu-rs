use crate::player::Player;
use crate::stage::Stage;
use crate::SharedGameState;

pub struct Frame {
    pub x: isize,
    pub y: isize,
    pub wait: isize,
}

impl Frame {
    pub fn update(&mut self, state: &SharedGameState, player: &Player, stage: &Stage) {
        if (stage.map.width - 1) * 16 < state.canvas_size.0 as usize {
            self.x = -(((state.canvas_size.0 as isize - ((stage.map.width - 1) * 16) as isize) * 0x200) / 2);
        } else {
            self.x += (player.target_x - (state.canvas_size.0 as isize * 0x200 / 2) - self.x) / self.wait;

            if self.x < 0 {
                self.x = 0;
            }

            let max_x = (((stage.map.width as isize - 1) * 16) - state.canvas_size.0 as isize) * 0x200;
            if self.x > max_x {
                self.x = max_x;
            }
        }

        if (stage.map.height - 1) * 16 < state.canvas_size.1 as usize {
            self.y = -(((state.canvas_size.1 as isize - ((stage.map.height - 1) * 16) as isize) * 0x200) / 2);
        } else {
            self.y += (player.target_y - (state.canvas_size.1 as isize * 0x200 / 2) - self.y) / self.wait;

            if self.y < 0 {
                self.y = 0;
            }

            let max_y = (((stage.map.height as isize - 1) * 16) - state.canvas_size.1 as isize) * 0x200;
            if self.y > max_y {
                self.y = max_y;
            }
        }

        // todo quake
    }
}
