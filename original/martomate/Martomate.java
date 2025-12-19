package martomate;

import java.awt.Color;
import java.awt.Dimension;
import java.awt.Font;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.Point;
import java.awt.Rectangle;
import java.awt.geom.Rectangle2D;
import java.awt.image.BufferedImage;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.net.MalformedURLException;
import java.net.URL;

import javax.swing.JFrame;
import javax.swing.JOptionPane;


public class Martomate extends JFrame {
	private static final long serialVersionUID = 1L;
	
	public static Tetris tetrisBackup;
	public static boolean backupSaved;
	
	private BufferedImage image;
	private Graphics2D graphics;
	
	private Button buttonPlay, buttonOptions, buttonTexture, buttonCredits, buttonQuit;
	
	private Input input;
	private boolean loadDone = false;
	
	public Martomate(){
		setTitle("The Martomate");
		setSize(400, 300);
		setResizable(false);
		setUndecorated(true);
		setLocationRelativeTo(null);
		setBackground(Color.BLACK);
		setVisible(true);
		
		Input.reset();
		input = new Input();
		addKeyListener(input);
		addFocusListener(input);
		addMouseListener(input);
		addMouseMotionListener(input);
		
		load();
		requestFocus();
		repaint();
	}// 						400 / 2 - (80 * 2 + 20) / 2
	
	public void load(){
		Dimension size1 = new Dimension(120, 45);
		Dimension size2 = new Dimension(80, 30);
		
		buttonPlay = new Button(getWidth() / 2 - size1.width / 2, 60, size1.width, size1.height, "Play!");
		buttonOptions = new Button(getWidth() / 2 - size2.width - 5, 150, size2.width, size2.height, "Options");
		buttonTexture = new Button(getWidth() / 2 + 5, 110, size2.width, size2.height, "Textures");
		buttonCredits = new Button(getWidth() / 2 - size2.width - 5, 110, size2.width, size2.height, "Credits");
		buttonQuit = new Button(getWidth() / 2 + 5, 150, size2.width, size2.height, "Quit");
		
//		File file = new File("%AppData%/Martomate/Martomate.jar");
//		if(!file.exists()){
//			forceUpdate();
//		}
		
		loadDone = true;
	}
	
	public void forceUpdate(){
		FileOutputStream writer;
		URL url;
	//	File file = new File("%Appdata%\\Martomate\\Martomate.jar");
		
		try {
			long startTime = System.currentTimeMillis();
			
			System.out.println("Connecting to download site...\n");
			
			url = new URL("http://www.metallica-guitar.coffeecup.com/backup/Martomate.jar");
			url.openConnection();
			InputStream reader = url.openStream();
			writer = new FileOutputStream("%Appdata%/Martomate/Martomate.jar");
			byte[] buffer = new byte[153600];
			int totalBytesRead = 0;
			int bytesRead = 0;
			
			System.out.println("Reading \"test.txt\" file 150KB blocks at a time.\n");
			
			while((bytesRead = reader.read(buffer)) > 0){
				writer.write(buffer, 0, bytesRead);
				buffer = new byte[153600];
				totalBytesRead += bytesRead;
				System.out.println((totalBytesRead / 1024) + "kB read at " + (int)((totalBytesRead / 1024.0) / (new Long(System.currentTimeMillis() - startTime).longValue() / 1000.0)) + " kB/s");
			}
			
			long endTime = System.currentTimeMillis();
			System.out.println("Done. " + (new Integer(totalBytesRead).intValue() / 1024) + " bytes read (" + (int)((totalBytesRead / 1024.0) / (new Long(endTime - startTime).longValue() / 1000.0)) + " kB/s).\n");
			writer.close();
			reader.close();
		}catch (MalformedURLException e) {
			System.out.println("Url-error: " + e.getMessage());
		}catch(IOException e){
			System.out.println("Connection-error: " + e.getMessage());
		}
	}
	
	public void paint(Graphics g1){
		Graphics2D g = (Graphics2D)g1;
		image = new BufferedImage(getWidth(), getHeight(), BufferedImage.TYPE_INT_RGB);
		graphics = image.createGraphics();
		
		
		paintComponent(graphics);
		g.drawImage(image, 0, 0, null);
	}
	
	public void paintComponent(Graphics2D g){
		g.setFont(new Font(Tetris.font, 0, 20));
		g.setColor(Color.BLACK);
		g.fillRect(0, 0, getWidth(), getHeight());
		g.setColor(new Color(0,0,255));
		g.drawRect(0, 0, getWidth() - 1, getHeight() - 1);
		
		g.setColor(new Color(0,150,0));
		g.setFont(new Font(Tetris.font, 0, 30));
		drawString(g, "The Martomate", getWidth() / 2 - (int)getTextBounds(g, "The Martomate").getWidth() / 2, 40);
		
		g.setFont(new Font(Tetris.font, 3, 14));
		drawString(g, "Martomate " + Tetris.version, 10, getHeight() - 10);
		
		if(loadDone){
			buttonPlay.draw(g);
			buttonOptions.draw(g);
			buttonTexture.draw(g);
			buttonCredits.draw(g);
			buttonQuit.draw(g);
			
			checkEvents(input.key);
		}
		
		try {
			Thread.sleep(1);
		} catch (InterruptedException e) {
			e.printStackTrace();
		}
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
		
		buttonPlay.update();
		buttonOptions.update();
		buttonTexture.update();
		buttonCredits.update();
		buttonQuit.update();
		
		if(buttonPlay.getDoAction()){
			dispose();
			new Tetris();
		}
		if(buttonOptions.getDoAction()){
		//	dispose();
		//	new Options();
		}
		if(buttonTexture.getDoAction()){
		//	dispose();
		//	new Texture();
		}
		if(buttonCredits.getDoAction()){
			dispose();
			new Credits();
		}
		if(buttonQuit.getDoAction()){
			System.exit(0);
		}
	}
	
	public static void setGameBackup(Tetris tetris){
		tetrisBackup = tetris;
	}
	
	public static void getGameBackup(Tetris tetris){
		tetris = tetrisBackup;
	}
	
	public static void drawString(Graphics2D g, String string, int x, int y){
		drawString(g, string, x, y, false);
	}
	
	public static void drawString(Graphics2D g, String string, int x, int y, boolean colorChange){
		Color color = (colorChange ? g.getColor() : new Color(0,255,0));
		
		g.setColor(new Color(color.getRed() / 5, color.getGreen() / 5, color.getBlue() / 5));
		
		g.drawString(string, x - 1, y - 1);
		g.drawString(string, x + 1, y - 1);
		g.drawString(string, x + 1, y + 1);
		g.drawString(string, x - 1, y + 1);
		
		g.drawString(string, x - 1, y);
		g.drawString(string, x, y - 1);
		g.drawString(string, x + 1, y);
		g.drawString(string, x, y + 1);
		
		g.setColor(color);
		g.drawString(string, x, y);
	}
	
	public static void showPopup(String message, String title){
		JOptionPane.showMessageDialog(null, message, title, JOptionPane.INFORMATION_MESSAGE);
	}
	
	public static void showPopup(Object obj, String title){
		JOptionPane.showMessageDialog(null, obj, title, JOptionPane.INFORMATION_MESSAGE);
	}
	
	public static boolean showConfirmPopup(String message, String title){
		int value = JOptionPane.showConfirmDialog(null, message, title, JOptionPane.YES_NO_OPTION);
		
		if(value == 0) return true;
		else return false;
	}
	
	public static Rectangle2D getTextBounds(Graphics2D g, String text){
		return (g.getFont().getStringBounds(text, g.getFontRenderContext()));
	}
	
	public static void main(String[] args){
		System.out.println("Now the program is going! :D");
		new Martomate();
	}
}
