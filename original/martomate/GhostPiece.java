package martomate;

import java.awt.Color;
import java.awt.Graphics2D;

public class GhostPiece extends Piece {

	public GhostPiece(int xpos, int ypos, int rotation, Shape shape) {
		super(xpos, ypos, rotation, shape);
	}
	
	public void draw(Graphics2D g){
		if(Tetris.ghostPieceEnabled && !Tetris.paused && shape != Shape.X){
			g.setColor(new Color(0, 0, 80, 200));
			for (int i = 0; i < 4; i++) {
				int x = xpos + x(i);
				int y = ypos + y(i);
				
				g.fillRect(x * Tetris.pieceSize, y * Tetris.pieceSize, Tetris.pieceSize, Tetris.pieceSize);
            }
		}
	}
	
	public void update(Tetris tetris){
		super.update(tetris);
	}
	
}
