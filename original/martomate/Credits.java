package martomate;
import java.awt.Color;
import java.awt.Font;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.Point;
import java.awt.Rectangle;
import java.awt.image.BufferedImage;

import javax.swing.JFrame;


public class Credits extends JFrame {
	private static final long serialVersionUID = 1L;
	
	public Input input;
	
	private Button quitButton;
	
	private BufferedImage image;
	private Graphics2D graphics;
	
	public Credits(){
		setSize(400, 300);
		setResizable(false);
		setUndecorated(true);
		setTitle("Credits");
		setLocationRelativeTo(null);
		setBackground(new Color(0,0,0));
		setVisible(true);
		
		Input.reset();
		input = new Input();
		addKeyListener(input);
		addFocusListener(input);
		addMouseListener(input);
		addMouseMotionListener(input);
		requestFocus();
		
		load();
		repaint();
	}
	
	public void load(){
		quitButton = new Button(20, getHeight() - 50, 80, 30, "Close");
	}
	
	public void paint(Graphics g1){
		Graphics2D g = (Graphics2D)g1;
		image = new BufferedImage(getWidth(), getHeight(), BufferedImage.TYPE_INT_RGB);
		graphics = image.createGraphics();
		
		paintComponent(graphics);
		g.drawImage(image, 0, 0, null);
		
		try {
			Thread.sleep(10);
		} catch (InterruptedException e) {
			e.printStackTrace();
		}
		
		repaint();
	}
	
	public void paintComponent(Graphics2D g){
		g.setFont(new Font(Tetris.font, 0, 20));
		g.setColor(new Color(0,0,0));
		g.fillRect(0, 0, getWidth(), getHeight());
		
		g.setColor(new Color(0,150,0));
		g.setFont(new Font(Tetris.font, 0, 30));
		Martomate.drawString(g, "The Martomate", 90, 40);
		
		g.setFont(new Font(Tetris.font, 2, 25));
		Martomate.drawString(g, "Program", 146, 80);
		
		g.setFont(new Font(Tetris.font, 0, 20));
		Martomate.drawString(g, "Martin Jakobsson", 114, 105);
		
		g.setFont(new Font(Tetris.font, 2, 25));
		Martomate.drawString(g, "Ideas", 166, 145);
		
		g.setFont(new Font(Tetris.font, 0, 20));
		Martomate.drawString(g, "Martin Jakobsson", 114, 165);
		Martomate.drawString(g, "Alexander Kirk", 128, 185);
		
		g.setFont(new Font(Tetris.font, 0, 24));
		Martomate.drawString(g, "Have fun :D", 133, 250);
		
		
		if(quitButton != null)
			quitButton.draw(g);
		
		checkEvents(input.key);
		
		g.setColor(new Color(0,0,255));
		g.drawRect(0, 0, getWidth() - 1, getHeight() - 1);
		
		repaint();
	}

	public void checkEvents(boolean key[]) {
		if(new Rectangle(0, 0, getWidth(), getHeight()).contains(new Point(Input.mousePosX, Input.mousePosY)) && Input.isClick){
			requestFocus();
		}
		
		if(!input.isFocused)
			return;
		
		if(Input.isDrag){
			Point location = getLocation();
			setLocation(location.x + Input.mouseDragX - Input.mouseStartX, location.y + Input.mouseDragY - Input.mouseStartY);
		}
		
		if(quitButton != null){
			quitButton.update();
			quitButton.update();
		}
		
		if(quitButton.getDoAction()){
			this.dispose();
			if(getParent() == null){
				new Martomate();
			}else{
				getParent().requestFocus();
			}
		}
	}
}
