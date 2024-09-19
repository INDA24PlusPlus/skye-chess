#[derive(Copy, Clone, PartialEq)]
enum ChessPieceKind {
    Rook,
    Pawn,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Copy, Clone, PartialEq)]
enum ChessColour {
    Black,
    White,
}

#[derive(Copy, Clone)]
pub struct ChessPiece {
    pos:u64,
    prev_pos: u64,
    colour: ChessColour,
    kind: ChessPieceKind,
    has_moved: bool,
    is_captured: bool,
    got_captured_now: bool;
}

fn get_rank(piece: ChessPiece)->u8{
    if piece.is_captured {
        return 0;
    }
    let mut scanner:u64=0xFF;
    for rank in 1..=8 {
        if(scanner & piece.pos)>0 {
            return 9-rank;
        }
        scanner=scanner<<8;
    }
    return 0;
}

fn get_file(piece: ChessPiece)->u8{
    if piece.is_captured {
            return 0;
    }
    let mut scanner = 0x8080808080808080;
    for file in 1..=8 {
        if(scanner & piece.pos)>0 {
            return file;
        }
        scanner=scanner>>1;
    }
    return 0;
}

fn get_colour_hash(col:ChessColour)->u8{
    return match col {
        ChessColour::White=>0b1,
        ChessColour::Black=>0b0,
    };
}

fn get_piece_hash(p_kind:ChessPieceKind)->u8{
    return match p_kind{
    ChessPieceKind::King=>0b000,
    ChessPieceKind::Queen=>0b001,
    ChessPieceKind::Rook=>0b010,
    ChessPieceKind::Bishop=>0b011,
    ChessPieceKind::Knight=>0b100,
    ChessPieceKind::Pawn=>0b101,

    };
}

fn hash_piece(piece: ChessPiece)->u16{
    let p_rank=get_rank(piece);
    let p_file=get_file(piece);
    let pos_hash=p_rank+(p_file << 3);
    let p_col=get_colour_hash(piece.colour);
    let p_kind=get_piece_hash(piece.kind);
    let mut out:u16=0;
    out+=pos_hash as u16;
    out=(out<<1)+(p_col as u16);
    out=(out<<3)+(p_kind as u16);
    return out;
}

#[derive(Copy, Clone)]
pub struct ChessBoard {
    pieces: [ChessPiece;32],
    current_move: ChessColour,
    rule_50_moves: u8,
    rule_repetition: [[u8;64];50]
}
impl ChessBoard{
    fn move_piece(&mut self, from_c:u64, to_c:u64)->bool{
        let mut other_o:Option<ChessPiece>=get_piece_bit_mask(from_c, *self);
        if other_o.is_none(){return false;}
        let piece:ChessPiece= other_o.take().unwrap();
        if (to_c&get_moves(piece, *self))==0{return false;}
        for mut piece_n in self.pieces{
            piece_n.prev_pos=piece_n.pos;
            if piece_n.pos==to_c{
                piece_n.is_captured=true;
            }
            if piece_n.pos==from_c{
                piece_n.pos=to_c;
            }
            
        }
        self.rule_50_moves+=1;
        self.current_move=get_op_col(self.current_move);
        return true;
    }

}



fn get_piece_map(col:ChessColour, board:ChessBoard)->u64{
    let mut out:u64=0x00;
    for piece in board.pieces{
        if piece.colour!=col||piece.is_captured{continue;}
        out=out|piece.pos;
    }
    return out;
}
fn get_all_piece_map(board:ChessBoard)->u64{
    let mut out:u64=0x00;
    for piece in board.pieces{
        if piece.is_captured{continue;}
        out=out|piece.pos;
    }
    return out;
}
fn get_op_col(col:ChessColour)->ChessColour{
    return match col {
        ChessColour::White=>ChessColour::Black,
        ChessColour::Black=>ChessColour::White,
    };
}
fn get_piece_bit_mask(pos:u64, board:ChessBoard)->Option<ChessPiece>{
    for piece in board.pieces{
        if (piece.pos&pos)>0{
            return Some(piece);
        }
    }
    return None;
}
fn hash_board_state(mut board: ChessBoard)->[u8;64]{
    let mut scanner:u64=0x01;
    let board_state:[u8;64]=[0;64];
    for mut square in board_state {
        let mut piece_opt=get_piece_bit_mask(scanner,board);
        if piece_opt.is_none(){square=0b0;continue;}
        let piece:ChessPiece= piece_opt.take().unwrap();
        square=((get_piece_hash(piece.kind)<<3)|get_colour_hash(piece.colour)<<1)|get_colour_hash(board.current_move);
        scanner=scanner<<1;
    }
    return board_state;
}
fn get_ep_capture_spots(col:ChessColour, board:ChessBoard)->u64{
    let mut out:u64=0x00;
    for piece in board.pieces{
        if (piece.colour==col)&&(piece.kind==ChessPieceKind::Pawn){
            if col==ChessColour::White{
                if (piece.pos>>16)==(piece.prev_pos){
                    out=out|(piece.pos>>8);
                }
            }
            if col==ChessColour::Black{
                if (piece.pos<<16)==(piece.prev_pos){
                    out=out|(piece.pos<<8);
                }
            }
        }
    }
    return out;

}

fn get_pawn_captures(piece: ChessPiece, _board:ChessBoard)->u64{
    if piece.colour==ChessColour::White{
        if get_file(piece)==1{return piece.pos<<7;}
        if get_file(piece)==8{return piece.pos<<9;}
        return (piece.pos<<7)|(piece.pos<<9);
    } 
    else if piece.colour==ChessColour::Black {
        if get_file(piece)==1{return piece.pos>>9;}
        if get_file(piece)==8{return piece.pos>>7;}
        return (piece.pos>>7)|(piece.pos>>9);
    }
    return 0x0;
}

fn get_pawn_moves(piece: ChessPiece,board:ChessBoard)->u64{
    let mut out:u64=0x0;
    
    if piece.colour==ChessColour::White{
        let capture_check:u64=get_pawn_captures(piece,board)&(get_piece_map(get_op_col(piece.colour), board)|get_ep_capture_spots(get_op_col(piece.colour),board));

        let normal_move_check:u64=(piece.pos<<8)&(!get_all_piece_map(board));
        let double_move_check:u64=((((0xFF00&piece.pos)<<8)&(!get_all_piece_map(board)))<<8)&(!get_all_piece_map(board));
        out=capture_check|normal_move_check|double_move_check;
    }
    else if piece.colour==ChessColour::Black{
        let capture_check:u64=get_pawn_captures(piece,board)&(get_piece_map(get_op_col(piece.colour), board)|get_ep_capture_spots(get_op_col(piece.colour),board));

        let normal_move_check:u64=(piece.pos>>8)&(!get_all_piece_map(board));
        let double_move_check:u64=((((0x00FF000000000000&piece.pos)>>8)&(!get_all_piece_map(board)))>>8)&(!get_all_piece_map(board));
        out=capture_check|normal_move_check|double_move_check;
    }
    return out;
}


fn get_knight_moves(piece: ChessPiece, board:ChessBoard)->u64{
    let mut out:u64=0x00;
    if get_file(piece)>=2{
        out=out|(piece.pos<<17)|(piece.pos>>15);
        if get_file(piece)>=3{
            out=out|(piece.pos>>6)|(piece.pos<<10);
        }
    }
    if get_file(piece)<=7{
        out=out|(piece.pos>>17)|(piece.pos<<15);
        if get_file(piece)<=6{
            out=out|(piece.pos>>10)|(piece.pos<<6);
        }
    }
    return out&(!get_piece_map(piece.colour,board));
}
fn get_rook_moves(piece: ChessPiece, board:ChessBoard)->u64{
    let capturable:u64=get_piece_map(get_op_col(piece.colour), board);
    let blockers:u64=get_piece_map(piece.colour, board);
    let rank:u8=get_rank(piece);
    let file:u8=get_file(piece);
    let left_move_max=rank-1;
    let right_move_max=8-rank;
    let up_move_max=file-1;
    let down_move_max=8-file;

    let mut out:u64=0x00;
    for offset in 1..=up_move_max{
        if ((piece.pos<<(8*offset))&(!blockers))>0{
            out=out|(piece.pos<<(8*offset));
            if ((piece.pos<<(8*offset))&capturable)>0{
                break;
            }
        }
        else {
            break;
        }
    }
    for offset in 1..=right_move_max{
        if ((piece.pos>>(offset))&(!blockers))>0{
            out=out|(piece.pos>>(offset));
            if ((piece.pos>>(offset))&capturable)>0{
                break;
            }
        }
        else {
            break;
        }
    }
    for offset in 1..=down_move_max{
        if ((piece.pos>>(8*offset))&(!blockers))>0{
            out=out|(piece.pos>>(8*offset));
            if ((piece.pos>>(8*offset))&capturable)>0{
                break;
            }
        }
        else {
            break;
        }
    }
    for offset in 1..=left_move_max{
        if ((piece.pos<<(offset))&(!blockers))>0{
            out=out|(piece.pos<<(offset));
            if ((piece.pos<<(offset))&capturable)>0{
                break;
            }
        }
        else {
            break;
        }
    }
    return out;
}
fn get_bishop_moves(piece:ChessPiece, board:ChessBoard)->u64{
    
    let capturable:u64=get_piece_map(get_op_col(piece.colour), board);
    let blockers:u64=get_piece_map(piece.colour, board);
    let rank:u8=get_rank(piece);
    let file:u8=get_file(piece);
    let left_move_max=rank-1;
    let right_move_max=8-rank;
    let up_move_max=file-1;
    let down_move_max=8-file;
    let ul_max=if up_move_max>left_move_max {left_move_max} else {up_move_max};
    let dl_max=if down_move_max>left_move_max {left_move_max} else {down_move_max};
    let ur_max=if up_move_max>right_move_max {right_move_max} else {up_move_max};
    let dr_max=if down_move_max>right_move_max {right_move_max} else {down_move_max};
    let mut out:u64=0x00;
    for offset in 1..=ul_max{
        if ((piece.pos<<(9*offset))&(!blockers))>0{
            out=out|(piece.pos<<(9*offset));
            if ((piece.pos<<(9*offset))&capturable)>0{
                break;
            }
        }
        else {
            break;
        }
    }
    for offset in 1..=ur_max{
        if ((piece.pos<<(7*offset))&(!blockers))>0{
            out=out|(piece.pos<<(7*offset));
            if ((piece.pos<<(7*offset))&capturable)>0{
                break;
            }
        }
        else {
            break;
        }
    }
    for offset in 1..=dl_max{
        if ((piece.pos>>(7*offset))&(!blockers))>0{
            out=out|(piece.pos>>(7*offset));
            if ((piece.pos>>(7*offset))&capturable)>0{
                break;
            }
        }
        else {
            break;
        }
    }
    for offset in 1..=dr_max{
        if ((piece.pos>>(9*offset))&(!blockers))>0{
            out=out|(piece.pos>>(9*offset));
            if ((piece.pos>>(9*offset))&capturable)>0{
                break;
            }
        }
        else {
            break;
        }
    }
    return out;
}
fn get_queen_moves(piece:ChessPiece, board:ChessBoard)->u64{
    return get_bishop_moves(piece,board)|get_rook_moves(piece, board);
}
fn get_king_moves_colour(col:ChessColour, board:ChessBoard)->u64{
    for piece in board.pieces{
        if piece.colour!=col{continue;}
        if piece.kind!=ChessPieceKind::King{continue;}
        if get_file(piece)==8{
            return ((piece.pos>>8)|(piece.pos>>7)|(piece.pos<<1)|(piece.pos<<9)|(piece.pos<<8))&(!(get_piece_map(col,board)));
        }
        if get_file(piece)==1{
            return ((piece.pos>>1)|(piece.pos>>9)|(piece.pos>>8)|(piece.pos<<8)|(piece.pos<<7))&(!(get_piece_map(col,board)));
        }
        return ((piece.pos>>1)|(piece.pos>>9)|(piece.pos>>8)|(piece.pos>>7)|(piece.pos<<1)|(piece.pos<<9)|(piece.pos<<8)|(piece.pos<<7))&(!(get_piece_map(col,board)));

    }
    return 0x00;
}

fn get_capture_map_king_check(col:ChessColour,board:ChessBoard)->u64{
    let mut out:u64=0x00;
    for piece in board.pieces{
        if piece.colour!=col{continue;}
        out=out|match piece.kind{
            ChessPieceKind::Rook=>get_rook_moves(piece, board),
            ChessPieceKind::Pawn=>get_pawn_captures(piece, board),
            ChessPieceKind::Knight=>get_knight_moves(piece, board),
            ChessPieceKind::King=>get_king_moves_colour(col,board),
            ChessPieceKind::Bishop=>get_bishop_moves(piece, board),
            ChessPieceKind::Queen=>get_queen_moves(piece, board),
        }
    }
    return out;
}

fn get_king_moves(piece: ChessPiece, board: ChessBoard)->u64{
    let mut out:u64=(piece.pos>>8)|(piece.pos<<8);
    let right_side:u64=(piece.pos>>1)|(piece.pos>>9)|(piece.pos<<7);
    let left_side:u64=(piece.pos>>7)|(piece.pos<<1)|(piece.pos<<9);
    if get_file(piece)!=1{out=out|left_side;}
    if get_file(piece)!=8{out=out|right_side;}
    return out&(!(get_piece_map(piece.colour,board)|get_capture_map_king_check(get_op_col(piece.colour), board)));
}

fn get_capture_map(col:ChessColour,board:ChessBoard)->u64{
    let mut out:u64=0x00;
    for piece in board.pieces{
        if piece.colour!=col{continue;}
        out=out|match piece.kind{
            ChessPieceKind::Rook=>get_rook_moves(piece, board),
            ChessPieceKind::Pawn=>get_pawn_captures(piece, board),
            ChessPieceKind::Knight=>get_knight_moves(piece, board),
            ChessPieceKind::King=>get_king_moves(piece,board),
            ChessPieceKind::Bishop=>get_bishop_moves(piece, board),
            ChessPieceKind::Queen=>get_queen_moves(piece, board),
        }
    }
    return out;
}
fn is_checked(col:ChessColour,board:ChessBoard)->bool{
    let check_map=get_capture_map_king_check(get_op_col(col), board);
    for piece in board.pieces{
        if piece.colour!=col{continue;}
        if piece.kind!=ChessPieceKind::King{continue;}
        return (piece.pos&check_map)>0;
    }
    return true;
}
fn get_long_castle_move(piece:ChessPiece, board:ChessBoard)->bool{
    if piece.has_moved{return false;}
    if is_checked(piece.colour,board){return false;}
    if piece.kind!=ChessPieceKind::King{return false}
    let mut other_o:Option<ChessPiece>=get_piece_bit_mask(piece.pos<<4, board);
    if other_o.is_none(){return false;}
    let other:ChessPiece= other_o.take().unwrap();
    if other.has_moved{return false;}
    if other.kind!=ChessPieceKind::Rook{return false;}
    let block_check_map=!(get_capture_map(get_op_col(piece.colour), board)|get_piece_map(piece.colour,board));
    return ((((((piece.pos<<1)&block_check_map)<<1)&block_check_map)<<1)&block_check_map)>0;        
}


fn get_short_castle_move(piece:ChessPiece, board:ChessBoard)->bool{
    if piece.has_moved{return false;}
    if is_checked(piece.colour,board){return false;}
    if piece.kind!=ChessPieceKind::King{return false}
    let mut other_o:Option<ChessPiece>=get_piece_bit_mask(piece.pos>>3, board);
    if other_o.is_none(){return false;}
    let other:ChessPiece= other_o.take().unwrap();
    if other.has_moved{return false;}
    if other.kind!=ChessPieceKind::Rook{return false;}
    let block_check_map=!(get_capture_map(get_op_col(piece.colour), board)|get_piece_map(piece.colour,board));
    return ((((piece.pos>>1)&block_check_map)>>1)&block_check_map)>0;
}




fn get_rank_u64(pos:u64)->u8{
    
    let mut scanner:u64=0xFF;
    for rank in 1..=8 {
        if(scanner & pos)>0 {
            return 9-rank;
        }
        scanner=scanner<<8;
    }
    return 0;
}

fn get_file_u64(pos:u64)->u8{
    let mut scanner = 0x8080808080808080;
    for file in 1..=8 {
        if(scanner & pos)>0 {
            return file;
        }
        scanner=scanner>>1;
    }
    return 0;
}

fn get_u64_pos(rank:u8, file:u8)->u64{
    return (0x1<<(8-file))<<(8*(8-rank));
}

pub fn get_moves(piece:ChessPiece, board:ChessBoard)->u64{
    return match piece.kind{
    ChessPieceKind::King=>get_king_moves(piece, board),
    ChessPieceKind::Queen=>get_queen_moves(piece, board),
    ChessPieceKind::Rook=>get_rook_moves(piece, board),
    ChessPieceKind::Bishop=>get_bishop_moves(piece, board),
    ChessPieceKind::Knight=>get_knight_moves(piece, board),
    ChessPieceKind::Pawn=>get_pawn_moves(piece, board),
    }
}
pub fn can_promote(piece:ChessPiece, board:ChessBoard)->bool{
    let mut other_o:Option<ChessPiece>=get_piece_bit_mask(from_c, board);
    if other_o.is_none(){return (board,false);}
    let piece:ChessPiece= other_o.take().unwrap();
    if piece.kind!=ChessPieceKind::Pawn{return false;}
    if piece.colour==ChessColour::Black{return (piece.pos&0xFF)>0;}
    return (piece.pos&0xFF00000000000000)>0;

}

pub fn move_piece(mut board:ChessBoard, from_c:u64, to_c:u64)->(ChessBoard,bool){
    let mut other_o:Option<ChessPiece>=get_piece_bit_mask(from_c, board);
    if other_o.is_none(){return (board,false);}
    let piece:ChessPiece= other_o.take().unwrap();
    if (to_c&get_moves(piece, board))==0{return (board, false);}
    let mut pieces=board.pieces;
    for mut piece_n in pieces{
        piece_n.prev_pos=piece_n.pos;
        if piece_n.pos==from_c{
            piece_n.pos=to_c;
        }
    }
    board.pieces=pieces;
    return (board, true);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_piece() {
        let test_piece=ChessPiece{
            pos:      0b0000000000000000000000000000000000000000000000000001000000000000,
            prev_pos: 0b0000000000000000000000000000000000000000000000000001000000000000,
            colour: ChessColour::Black,
            kind: ChessPieceKind::Queen,
            has_moved:false,
            is_captured:false,
        };
        assert_eq!(get_rank(test_piece),7);
        assert_eq!(get_file(test_piece),4);
        assert_eq!(hash_piece(test_piece),0b0);
    }
    #[test]
    fn test_edge_right(){
        let pos=0b0000000000000000000000000000000100000000000000000000000000000000;
        assert_eq!(get_rank_u64(pos),4);
        assert_eq!(get_file_u64(pos),8);
    }
    #[test]
    fn test_edge_left(){
        let pos= 0b0000000000000000000000001000000000000000000000000000000000000000;
        assert_eq!(get_rank_u64(pos),4);
        assert_eq!(get_file_u64(pos),1);
    }
    #[test]
    fn test_no_pos(){
        let pos= 0b0000000000000000000000000000000000000000000000000000000000000000;
        assert_eq!(get_rank_u64(pos),0);
    }
}
