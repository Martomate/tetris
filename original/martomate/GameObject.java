package martomate;
import java.awt.Graphics2D;


public abstract class GameObject {
	
	protected Shape shape;
	protected int[][] coords;
	protected int xpos, ypos, rotation;
	
	
	abstract void draw(Graphics2D g);
	
	abstract void update(Tetris tetris);
}
