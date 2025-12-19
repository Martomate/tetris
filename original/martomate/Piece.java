package martomate;
import java.awt.Graphics2D;


public class Piece extends GameObject{
	
	public int[][][] coordsCollection;
	public int[][] coords = new int[4][2];
	private Tetris tetris;
	public float moved;
	
	public Piece(final int xpos, final int ypos, final int rotation, final Shape shape){
		this.xpos = xpos;
		this.ypos = ypos;
		this.rotation = rotation;
		this.shape = shape;
		this.setShape();
		moved = 0.0f;
	}
	
	public void draw(Graphics2D g){
		if (shape != Shape.X) {
			for (int i = 0; i < 4; i++) {
				int x = xpos + x(i);
				int y = ypos + y(i);
				
				g.drawImage(Tetris.images_shapes[Tetris.curShape.ordinal()][(rotation) % 4][i], x * Tetris.pieceSize, (int)((y + moved) * Tetris.pieceSize), Tetris.pieceSize, Tetris.pieceSize, null);
            }
		}
	}
	
	public static void drawFallenPieces(Graphics2D g){
		for(int i = 0; i < Tetris.board.length; i++){
			for(int j = 0; j < Tetris.board[i].length; j++){
				if(Tetris.board[i][j] != Tetris.images_shapes[7][0][0] && !Tetris.paused){
					g.drawImage(Tetris.board[i][j], i * Tetris.pieceSize, j * Tetris.pieceSize, Tetris.pieceSize, Tetris.pieceSize, null);
				}
			}
		}
	}
	
	void update(Tetris tetris) {
		this.tetris = tetris;
	}
	
	public void setShape(){
		coordsCollection = new int[][][] {
				{ { 0,-1 },	{ 0, 0 },	{ 1, 0 },	{ 1,-1 } }, // O
				{ { 0,-1 },	{ 0, 0 },	{ 0, 1 }, 	{ 0, 2 } }, // I
				{ { 1,-1 },	{ 1, 0 },	{ 1, 1 },	{ 0, 1 } }, // J
				{ { 0,-1 },	{ 0, 0 },	{ 0, 1 },	{ 1, 1 } }, // L
				{ { 1,-1 },	{ 1, 0 },	{ 0, 0 },	{ 0, 1 } }, // Z
				{ { 0,-1 },	{ 0, 0 },	{ 1, 0 },	{ 1, 1 } }, // S
				{ { 0,-1 },	{ 0, 0 },	{ 0, 1 },	{ 1, 0 } }, // T
				{ { 0, 0 },	{ 0, 0 },	{ 0, 0 },	{ 0, 0 } }
		};
		
		for(int i = 0; i < 4; i++){
			for(int j = 0; j < 2; ++j){
				coords[i][j] = coordsCollection[shape.ordinal()][i][j];
			}
		}
	}
	
	public void setX(int i, int x){
		coords[i][0] = x;
	}
	public void setY(int i, int y){
		coords[i][1] = y;
	}
	public int x(int i){
		return coords[i][0];
	}
	public int y(int i){
		return coords[i][1];
	}
	
	public int minX(){
		int m = coords[0][0];
		for (int i = 0; i < 4; i++) {
			m = Math.min(m, coords[i][0]);
		}
		return m;
	}
	public int minY(){
		int m = coords[0][1];
		for (int i = 0; i < 4; i++) {
			m = Math.min(m, coords[i][1]);
		}
		return m;
	}
	public int maxX(){
		int m = coords[0][0];
		for (int i = 0; i < 4; i++) {
			m = Math.max(m, coords[i][0]);
		}
		return m;
	}
	public int maxY(){
		int m = coords[0][1];
		for (int i = 0; i < 4; i++) {
			m = Math.max(m, coords[i][1]);
		}
		return m;
	}
	
	public void rotateRight(){
		Piece pieze2 = new Piece(xpos, ypos, rotation, shape);
		
		for (int i = 0; i < 4; ++i) {
			pieze2.setX(i, -y(i));
			pieze2.setY(i, x(i));
		}
		
		if(tetris.movePieze(pieze2, 0, 0)){
			this.coords = pieze2.coords;
		}
	}
	
	public void rotateLeft(){
		Piece pieze2 = new Piece(xpos, ypos, rotation, shape);
		
		for (int i = 0; i < 4; ++i) {
			pieze2.setX(i, y(i));
			pieze2.setY(i, -x(i));
		}
		
		if(tetris.movePieze(pieze2, 0, 0)){
			this.coords = pieze2.coords;
		}
	}
	
	public String toString(){
		return "xpos: " + xpos + "\nypos: " + ypos + "\nrotation: " + rotation + "shape: " + shape;
	}
}
