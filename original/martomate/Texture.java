package martomate;

import java.awt.Color;
import java.awt.Font;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.Image;
import java.awt.Point;
import java.awt.Rectangle;
import java.awt.event.MouseWheelEvent;
import java.awt.event.MouseWheelListener;
import java.awt.image.BufferedImage;
import java.awt.image.CropImageFilter;
import java.awt.image.FilteredImageSource;
import java.io.File;
import java.io.IOException;
import java.util.Scanner;
import java.util.Vector;

import javax.imageio.ImageIO;
import javax.swing.ImageIcon;
import javax.swing.JFrame;

public class Texture extends JFrame implements Runnable, MouseWheelListener {
	private static final long serialVersionUID = 1L;
	
	public static int curTexture = 0;
	
	private int frameWidth, frameHeight, scroll = -40;
	
	private Vector<TextureRect> texturepacks = new Vector<TextureRect>();
	private ScrollBar scrollbar;
	private Button quitButton;
	
	private BufferedImage image;
	private Graphics2D graphics;
	private Thread thread;
	
	private Input input;
	
	private boolean running = false;
	
	public Texture() {
		setTitle("Textures");
		setSize(600, 400);
		setResizable(false);
		setUndecorated(true);
		setLocationRelativeTo(null);
		setLocation(getLocation().x * 2, 0);
		setBackground(new Color(0, 0, 0));
		setVisible(true);
		
		Input.reset();
		input = new Input();
		addKeyListener(input);
		addFocusListener(input);
		addMouseListener(input);
		addMouseMotionListener(input);
		addMouseWheelListener(this);
		
		load();
		requestFocus();
		start();
	}
	
	public void load() {
		quitButton = new Button(20, getHeight() - 40, 80, 30, "Close", new Color(150,150,150));
		
		frameWidth = getWidth();
		frameHeight = getHeight();
		
		scrollbar = new ScrollBar();
		
		File file = new File("res/Texturepacks");
		for(File f : file.listFiles()){
			if(f.isDirectory()){
				texturepacks.add(new TextureRect(texturepacks.size(), "res/Texturepacks/" + f.getName()));
			}
		}
		
		texturepacks.get(curTexture).isCurrent = true;
		
		for (int i = 0; i < texturepacks.size(); i++) {
			if(texturepacks.get(i) != null)
				texturepacks.get(i).load();
		}
	}
	
	public void paint(Graphics g1) {
		Graphics2D g = (Graphics2D) g1;
		image = new BufferedImage(getWidth(), getHeight(), BufferedImage.TYPE_INT_RGB);
		graphics = image.createGraphics();
		
		paintComponent(graphics);
		g.drawImage(image, 0, 0, null);
	}
	
	public void paintComponent(Graphics2D g) {
		g.setColor(new Color(0, 0, 0));
		g.fillRect(0, 0, getWidth(), getHeight());
		
		for (int i = 0; i < texturepacks.size(); i++) {
			if (texturepacks.get(i) != null)
				texturepacks.get(i).draw(g);
		}
		
		g.setColor(Color.BLACK);
		g.fillRect(0, 0, getWidth(), 10);
		g.fillRect(0, getHeight() - 10, getWidth(), 10);
		
		scrollbar.draw(g);
		
		if (quitButton != null)
			quitButton.draw(g);
		
		g.setColor(new Color(0, 0, 255));
		g.drawRect(0, 0, getWidth() - 1, getHeight() - 1);
	}

	
	public void scroll(int value){
		scroll += value;
	}
	
	public void run() {
		while (running) {
			try {
				Thread.sleep(10);
			} catch (InterruptedException e) {
				e.printStackTrace();
			}
			
			repaint();
			checkEvents(input.key);
		}
	}
	
	public void start() {
		if (running)
			return;
		running = true;
		thread = new Thread(this, "Texture");
		thread.start();
	}
	
	public void stop() {
		if (!running)
			return;
		running = false;
		try {
			thread.join();
		} catch (InterruptedException e) {
			e.printStackTrace();
		}
	}
	
	public void checkEvents(boolean[] key) {
		if (new Rectangle(0, 0, getWidth(), getHeight()).contains(new Point(Input.mousePosX, Input.mousePosY)) && Input.isClick) {
			requestFocus();
		}
		
		if (!input.isFocused)
			return;
		
		int msx = Input.mousePosX;
		int msy = Input.mousePosY;
		
		if (Input.isDrag) {
			Point location = getLocation();
			setLocation(location.x + Input.mouseDragX - Input.mouseStartX, location.y + Input.mouseDragY - Input.mouseStartY);
		}
		
		scrollbar.events();
		
		if (quitButton != null) {
			quitButton.update();
			quitButton.update();
		}
		
		if (quitButton.getDoAction()) {
			this.dispose();
			if (getParent() == null) {
				new Martomate();
			} else {
				getParent().requestFocus();
			}
		}
		
		for (int i = 0; i < texturepacks.size(); i++) {
			if (texturepacks.get(i) == null)
				continue;
			if (new Rectangle(texturepacks.get(i).x, texturepacks.get(i).y - scroll, texturepacks.get(i).width, texturepacks.get(i).height).contains(new Point(msx, msy))) {
				texturepacks.get(i).isHovered = true;
				if (Input.isClick) {
					texturepacks.get(i).isLoading = true;
					repaint();
					setTexturePack(texturepacks.get(i));
					for (int j = 0; j < texturepacks.size(); j++) {
						if (j != i) {
							texturepacks.get(j).isCurrent = false;
						} else {
							texturepacks.get(j).isCurrent = true;
							curTexture = j;
						}
					}
					texturepacks.get(i).isLoading = false;
					Input.isClick = false;
				}
			} else {
				texturepacks.get(i).isHovered = false;
			}
		}
	}
	
	public static void saveImage(BufferedImage img, String path) {
		try {
			ImageIO.write(img, path, new File(path));
		} catch (IOException ex) {
			ex.printStackTrace();
		}
	}
	
	public static Color getPixelColor(BufferedImage img, int x, int y) {
		int colorBits = img.getRGB(x, y);
		
		int red = (colorBits & 0xff0000) >> 16;
		int green = (colorBits & 0x00ff00) >> 8;
		int blue = (colorBits & 0x0000ff);
		
		return new Color(red, green, blue);
	}
	
	public void setTexturePack(TextureRect txtr) {
		String path = txtr.path;
		try {
			txtr.isLoading = true;
			for (int s = 0; s < Shape.X.ordinal(); s++) {
				for (int r = 0; r < 4; r++) {
					for (int p = 0; p < 4; p++) {
						Tetris.images_shapes[s][r][p] = new ImageIcon(Tetris.loadImage(path + "/" + Tetris.getShapeName(s) + "." + Tetris.image_type)).getImage();
						Tetris.images_shapes[s][r][p] = createImage(new FilteredImageSource(Tetris.images_shapes[s][r][p].getSource(), new CropImageFilter(Tetris.image_resolution * p,
								Tetris.image_resolution * r, Tetris.image_resolution, Tetris.image_resolution)));
						repaint();
					}
				}
			}
		} catch (Exception e) {
			e.printStackTrace();
			//	System.out.println("Error in Texture.setTexturePack()! Check path:\t" + path);
		} finally {
			txtr.isLoading = false;
		}
	}
	
	public void mouseWheelMoved(MouseWheelEvent e) {
		if(e.getWheelRotation() > 0){
			scroll(16);
		}else if(e.getWheelRotation() < 0){
			scroll(-16);
		}
		
		int totalHeight = (texturepacks.get(texturepacks.size() - 1).y + texturepacks.get(texturepacks.size() - 1).height) - texturepacks.get(0).y;
		
		if(scroll > ((-getHeight() / 2) + totalHeight)){
			scroll = (-getHeight() / 2) + totalHeight;
		}
		if(scroll < -getHeight() / 2){
			scroll = -getHeight() / 2;
		}
	}
	
	
	class TextureRect extends Rectangle {
		private static final long serialVersionUID = 1L;
		
		public boolean isHovered = false;
		public boolean isLoading = false;
		public boolean isCurrent = false;
		
		public String path;
		private String description = "";
		private String name = "";
		private int colorUp = 80;
		private int time = 0;
		
		private Image img;
		
		private Scanner sc;
		
		public TextureRect(int index, String path) {
			setBounds(120, index * 100, frameWidth - 160, 90);
			this.path = path;
		}
		
		public void load() {
			try {
				sc = new Scanner(Texture.class.getResourceAsStream(path + "/TextureInfo.txt"));
				img = Tetris.loadImage(path + "/Logo.jpg");
			} catch (Exception e) {
				for (int i = 0; i < texturepacks.size(); i++) {
					if (texturepacks.get(i) != null && texturepacks.get(i).path == path) {
						for (int j = i; j < texturepacks.size(); j++) {
							if (j < texturepacks.size() - 1) {
								texturepacks.set(j, texturepacks.get(j + 1));
								continue;
							}
							texturepacks.set(j, null);
						}
						return;
					}
				}
				System.out.println("Error in TextureRect.load()! Check path:\t" + path);
			}
			while (sc.hasNext()) {
				String line = sc.nextLine();
				int colonPos = line.indexOf(':');
				String str1 = line.substring(0, colonPos);
				String str2 = line.substring(colonPos + 1, line.length());
				
				if (str1.equals(new String("name"))) {
					name = str2;
				} else if (str1.equals("description")) {
					description = str2;
				}
			}
		}
		
		public void draw(Graphics2D g) {
			drawBorder(g);
			
			if (img != null) {
				g.drawImage(img, x + 13, y + 13 - scroll, 64, 64, null);
				g.setColor(new Color(0, 0, 0, 255 - colorUp));
				g.fillRect(x + 13, y + 13 - scroll, 64, 64);
			}
			g.setColor(new Color(20, 20, 20));
			g.drawRect(x + 12, y + 12 - scroll, 65, 65);
			
			int textOff = 85;
			g.setColor(new Color(50 + colorUp, 50 + colorUp, 50 + colorUp));
			g.setFont(new Font(Tetris.font, 0, 20));
			Martomate.drawString(g, name, x + textOff, y + 30 - scroll, true);
			g.setFont(new Font(Tetris.font, 0, 16));
			int descWidth = (int) Martomate.getTextBounds(g, description).getWidth();
			if ((descWidth + textOff) > (width - 10)) {
				int warpPos = description.length();
				int lastSpacePos = -1;
				for (int i = 0; i < description.length(); i++) {
					int spacePos = description.substring(lastSpacePos + 1, i).indexOf(' ') + lastSpacePos + 1;
					if (spacePos > lastSpacePos) {
						lastSpacePos = spacePos;
					}
					int spacePosAdd = Math.max(description.substring(spacePos + 1).indexOf(' ') + 1, 0);
					int spacePosAddWidth = (int) (Martomate.getTextBounds(g, description.substring(0, spacePos + spacePosAdd + 1)).getWidth() + textOff);
					int spacePosAdd2 = (spacePosAdd > 0 ? Math.max(description.substring(spacePos + spacePosAdd + 1).indexOf(' ') + 1, 0) : 0);
					int spacePosAddWidth2 = (int) (Martomate.getTextBounds(g, description.substring(0, spacePos + spacePosAdd2 + 1)).getWidth() + textOff);
					
					if ((spacePosAddWidth) > (width - 10) || (spacePosAddWidth2) > (width - 10)) {
						warpPos = i;
						break;
					}
				}
				Martomate.drawString(g, description.substring(0, warpPos), x + textOff, y + 50 - scroll, true);
				Martomate.drawString(g, description.substring(warpPos), x + textOff, y + 70 - scroll, true);
			} else {
				Martomate.drawString(g, description, x + textOff, y + 50 - scroll, true);
			}
			
			if (isLoading) {
				drawLoading(g);
				time++;
			} else {
				time = 0;
			}
		}
		
		private void drawBorder(Graphics2D g) {
			int size = 5; // how wide the border is / 2
			if (isCurrent)
				colorUp = 160;
			else if (isHovered)
				colorUp = 130;
			else
				colorUp = 80;
			
			width--;
			height--;
			
			g.setColor(new Color(50 + colorUp / 5, 50 + colorUp / 5, 50 + colorUp / 5));
			g.fillRect(x, y - scroll, width, height);
			
			for (int i = 0; i < size; i++) {
				int color = 0 + i * (colorUp / size);
				
				if (color > 255)
					System.out.println(color + " is more than 255!! (Texture:396)");
				g.setColor(new Color(Math.max(Math.min(color, 255), 0), Math.max(Math.min(color, 255), 0), Math.max(Math.min(color, 255), 0)));
				g.drawRect(x + i, y + i - scroll, width - i * 2, height - i * 2);
			}
			for (int i = 0; i < size; i++) {
				int color = colorUp - i * (colorUp / size) / 2;
				
				if (color > 255)
					System.out.println(color + " is more than 255!! (Texture:404)");
				g.setColor(new Color(Math.max(Math.min(color, 255), 0), Math.max(Math.min(color, 255), 0), Math.max(Math.min(color, 255), 0)));
				g.drawRect(x + size + i, y + size + i - scroll, width - i * 2 - size * 2, height - i * 2 - size * 2);
			}
			
			width++;
			height++;
		}
		
		private void drawLoading(Graphics2D g) {
			int bla = 10;
			for (int i = 1; i <= bla; i++) {
				Point p1 = new Point((int) (Math.cos(1 / 20.0 + (Math.PI * 2) / bla * i) * 20.0), (int) (Math.sin(1 / 20.0 + (Math.PI * 2) / bla * i) * 20.0));
				Point p2 = new Point((int) (Math.cos(1 / 20.0 + (Math.PI * 2) / bla * i + Math.PI / 10.0) * 20.0), (int) (Math.sin(1 / 20.0 + (Math.PI * 2) / bla * i + Math.PI / 10.0) * 20.0));
				
				int red = Math.min(Math.max(100 + (int) (Math.min(Math.cos(time / 10.0 - (Math.PI * 2) / bla * i) * 50.0, Math.sin(time / 10.0 - (Math.PI * 2) / bla * i) * 50.0)), 50 + colorUp / 5),
						255);
				int green = Math.min(
						Math.max(100 + (int) (Math.min(Math.cos(time / 10.0 - (Math.PI * 2) / bla * i) * 50.0, Math.sin(time / 10.0 - (Math.PI * 2) / bla * i) * 50.0)), 50 + colorUp / 5), 255);
				int blue = Math.min(Math.max(100 + (int) (Math.min(Math.cos(time / 10.0 - (Math.PI * 2) / bla * i) * 50.0, Math.sin(time / 10.0 - (Math.PI * 2) / bla * i) * 50.0)), 50 + colorUp / 5),
						255);
				
				g.setColor(new Color(red, green, blue));
				g.fillPolygon(new int[] { (x + width - 38) + p1.x, (x + width - 38) + p2.x, (x + width - 38) + p2.x / 2, (x + width - 38) + p1.x / 2 }, new int[] { (y + height - 38) + p1.y - scroll,
						(y + height - 38) + p2.y - scroll, (y + height - 38) + p2.y / 2 - scroll, (y + height - 38) + p1.y / 2 - scroll }, 4);
			}
		}
	}
	
	class ScrollBar {
		private int light[] = new int[3];
		private boolean loadScroll = false;
		
		private Rectangle[] buttons = new Rectangle[3];
		
		public ScrollBar(){
			
			
			buttons[0] = new Rectangle(frameWidth - 30, 10, 20, 20);
			buttons[1] = new Rectangle(frameWidth - 30, frameHeight - 30, 20, 20);
			buttons[2] = new Rectangle(buttons[0].x, buttons[0].y + buttons[0].height, buttons[0].width, buttons[1].y - (buttons[0].y + buttons[0].height));
			for(int i = 0; i < 3; i++){
				light[i] = 15;
			}
		}
		
		public void draw(Graphics2D g){
			for(int i = 0; i < 3; i++){
				drawRect(g, i);
			}
		}
		
		private void drawRect(Graphics2D g, int index){
			for(int i = 0; i < 5; i++){
				g.setColor(new Color(50 + i * light[index], 50 + i * light[index], 50 + i * light[index]));
				g.fillRoundRect(buttons[index].x + i, buttons[index].y + i, buttons[index].width - i * 2, buttons[index].height - i * 2, 5, 5);
				g.drawRoundRect(buttons[index].x + i, buttons[index].y + i, buttons[index].width - i * 2, buttons[index].height - i * 2, 5, 5);
			}
		}
		
		public void events(){
			Point mouse = new Point(Input.mousePosX, Input.mousePosY); 
			for(int i = 0; i < 3; i++){
				if(buttons[i].contains(mouse)){
					light[i] = 20;
					if(Input.isClick){
						light[i] = 10;
						loadScroll = false;
					}else{
						if(loadScroll){
							if(i == 0){
								scroll(16);
							}else if(i == 2){
								scroll(-16);
							}else if(i == 1){
								
							}
							loadScroll = false;
						}
					}
				}else{
					light[i] = 15;
				}
			}
		}
	}
}
