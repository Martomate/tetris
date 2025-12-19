package martomate;

import java.awt.Color;
import java.awt.Font;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.Rectangle;

public class SideBar {
	
	public Rectangle mainRect;
	
	public int x, y, width, height;
	
	public SideBar(int x, int y, int width, int height) {
		this.x = x;
		this.y = y;
		this.width = width;
		this.height = height;
		
		this.mainRect = new Rectangle(x, y, width, height);
		
		//load();
	}
	
	public void load() {
		
	}
	
	public void draw(Graphics g1, Tetris tetris) {
		Graphics2D g = (Graphics2D) g1;
		g.setFont(new Font(Tetris.font, 0, 20));
		g.setColor(new Color(0, 0, 50));
		g.fillRect(mainRect.x, mainRect.y, mainRect.width, mainRect.height);
		
		drawHole(g, mainRect.x + 40, mainRect.y + 40, mainRect.width - 80, mainRect.width - 50);
		
		g.setColor(new Color(0, 150, 0));
		Martomate.drawString(g, "Next Piece", mainRect.x + 30, mainRect.y + 30);
		
		if (tetris.piece != null) {
			Piece pieze2 = new Piece(Tetris.board.length / 2 - 1, tetris.piece.ypos, 0, Tetris.nextShape);
			int viewPiezeSize = mainRect.width / 8;
			for (int i = 0; i < 4; ++i) {
				int x = pieze2.x(i);
				int y = pieze2.y(i);
				int h = (pieze2.maxY() - pieze2.minY()) + 1;
				int w = (pieze2.maxX() - pieze2.minX()) + 1;
				
				int xx = mainRect.x + (mainRect.width / 2 - (w * viewPiezeSize) / 2 + (x * viewPiezeSize));
				int yy = mainRect.y + (mainRect.width / 2 - (h * viewPiezeSize) / 2 + (y * viewPiezeSize));
				
				g.drawImage(Tetris.images_shapes[pieze2.shape.ordinal()][pieze2.rotation][i], xx, 35 + yy, viewPiezeSize, viewPiezeSize, null);
			}
		}
		
		Martomate.drawString(g, "Score: " + (tetris.running == true ? tetris.score : ""), mainRect.x + 20, mainRect.width + 45 + 15);
		Martomate.drawString(g, "Lines: " + (tetris.running == true ? tetris.linesRemoved : ""), mainRect.x + 20, mainRect.width + 45 + 45);
		Martomate.drawString(g, "Level: " + (tetris.running == true ? tetris.level : ""), mainRect.x + 20, mainRect.width + 45 + 75);
		
		drawControls(g);
	}
	
	public void checkEvents(Tetris tetris, boolean key[]) {
		
	}
	
	public void drawControls(Graphics2D g) {
		g.setFont(new Font(g.getFont().getFamily(), g.getFont().getStyle(), g.getFont().getSize() / 4 * 3));
		
		String[][] controls = new String[][] { { "Rotate", "UP" }, { "Left", "LEFT" }, { "Right", "RIGHT" }, { "Down", "DOWN" }, { "Drop", "D" }, { "Pause", "P" }, { "Exit", "ESC" } };
		
		int yOff = mainRect.height - (controls.length * 20 + 10);
		
		drawHole(g, mainRect.x + 10, (yOff - 25), mainRect.width - 20, controls.length * 20 + 5 + 20);
		
		for (int i = 0; i < controls.length; i++) {
			Martomate.drawString(g, controls[i][0], mainRect.x + 20, yOff + i * 20 + (i > 3 ? 5 : 0));
		}
		
		for (int i = 0; i < controls.length; i++) {
			double stringWidth = Martomate.getTextBounds(g, controls[i][1]).getWidth();
			Martomate.drawString(g, controls[i][1], mainRect.x + mainRect.width - 20 - (int) stringWidth, yOff + i * 20 + (i > 3 ? 5 : 0));
		}
		
		drawHeader(g, mainRect.x + mainRect.width / 2 - (int) (Martomate.getTextBounds(g, "Controls").getWidth() / 2) - 10, yOff - 30 - (int) (Martomate.getTextBounds(g, "Controls").getHeight()) - 2,
				(int) (Martomate.getTextBounds(g, "Controls").getWidth() + 20), (int) (Martomate.getTextBounds(g, "Controls").getHeight() + 16));
		
		Martomate.drawString(g, "Controls", mainRect.x + mainRect.width / 2 - (int) (Martomate.getTextBounds(g, "Controls").getWidth() / 2), yOff - 25);
		
	}
	
	public void drawHole(Graphics2D g, int x, int y, int width, int height) {
		int size = 20;
		for (int i = 0; i < size; i++) {
			g.setColor(new Color(0, 0, 50 - (int) (Math.sin((Math.PI / 2 / size) * i) * 50)));
			g.fillRect(x + i, y + i, width - i * 2, height - i * 2);
		}
	}
	
	public void drawHeader(Graphics2D g, int x, int y, int width, int height) {
		int size = 20;
		for (int i = 0; i < size; i++) {
			g.setColor(new Color(0, 0, 50 - (int) (Math.sin((Math.PI / 2 / size) * i) * 50)));
			g.fillRect(x + i, y + i, width - i * 2, height - i * 2);
		}
		
		for (int i = 0; i < size; i++) {
			g.setColor(new Color(0, 0, 5));
			g.drawLine(x + size, y + height - size + 1 + i, x - size + width, y + height - size + 1 + i);
			
			g.setColor(new Color(0, 0, 50 - (int) (Math.sin((Math.PI / 2 / size) * i) * 50)));
			
			size--;
			g.drawLine(x, y + height - size + i, x + i, y + height - size + i);
			g.drawLine(x + i, y + height - size, x + i, y + height - size + i);
			
			x--;
			g.drawLine(x + width, y + height - size + i, x + width - i, y + height - size + i);
			g.drawLine(x + width - i, y + height - size, x + width - i, y + height - size + i);
			x++;
			
			size++;
		}
	}
}
