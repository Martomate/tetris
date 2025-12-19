package martomate;

import java.awt.Color;
import java.awt.Font;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.Image;
import java.awt.Point;
import java.awt.Rectangle;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.awt.event.KeyEvent;
import java.awt.image.BufferedImage;
import java.awt.image.CropImageFilter;
import java.awt.image.FilteredImageSource;
import java.text.SimpleDateFormat;
import java.util.Date;
import java.util.Random;

import javax.imageio.ImageIO;
import javax.swing.ImageIcon;
import javax.swing.JFrame;
import javax.swing.Timer;

public class Tetris extends JFrame implements Runnable {
	private static final long serialVersionUID = 1L;
	
	/** 	 ________________________________________README_____________________________________________
	 * 		|																							|
	 * 		|	This is just a message from me, the author of The Martomate, to you (a random person..)	|
	 * 		|	Just so that you know, I like to spell piece with a Z (pieze). 							|
	 * 		|	I don't want you to go mad about that so I made every piece word spell with a Z. 		|
	 * 		|	And by the way, how can you see my code???? ...I'm just wondering. 						|
	 * 		|	Hmm...																					|
	 * 		|	Well then I think I have to say: Have fun with the code and think before you edit ;)	|
	 * 		|___________________________________________________________________________________________|
	**/
	
	public static final String version = "Alpha 1.2";
	
	public static int WIDTH = 320; //10 * 32
	public static int HEIGHT = 640; //20 * 32
	public static int image_resolution = 64;
	public static int pieceSize = WIDTH / 10;
	
	public static String image_type = "png";
	public static String font = "Copper Black";
	public static Image[][][] images_shapes = new Image[8][4][4]; //8 pieces, 4 rotations, 4 parts
	public static Image[][] board = new Image[10][20];
	
	public int linesRemoved = 0;
	public int score = 0;
	public int level = 0;
	
	public static boolean isDebug = false;
	public static boolean ghostPieceEnabled = true;
	public static boolean paused = false;
	public static boolean hasWon = false;
	
	public static Shape curShape = Shape.X;
	public static Shape nextShape = Shape.X;
	public Piece piece;
	public Piece ghostPiece;
	
	private SideBar sideBar;
	private Input input;
	
	private BufferedImage image;
	private Graphics graphics;
	private Random random = new Random();
	private Thread thread;
	private Timer timer;
	
	public boolean running = false;
	private boolean rotating = false;
	private boolean loadStarted = false;
	private boolean loadDone = false;
	public boolean gameOver = false;
	private boolean forceScreenShot = false;
	
	private int levelsToWin = 60;
	
	public Tetris() {
		setTitle("The Martomate");
		setResizable(false);
		setSize(WIDTH + WIDTH / 2, HEIGHT);
		setUndecorated(true);
		setLocationRelativeTo(null);
		setBackground(new Color(0, 0, 20));
		setDefaultCloseOperation(EXIT_ON_CLOSE);
		setVisible(true);
		
		Input.reset();
		input = new Input();
		addKeyListener(input);
		addMouseListener(input);
		addMouseMotionListener(input);
		addFocusListener(input);
		
		loadDone = false;
		repaint();
	}
	
	public synchronized void load(Graphics g) {
		if (loadDone || loadStarted) {
			return;
		} else {
			loadStarted = true;
			g.setFont(new Font(font, 0, 20));
			
			// s = shape number		r = rotation number		p = part
			for (int s = 0; s <= Shape.X.ordinal(); s++) {
				for (int r = 0; r < 4; r++) {
					g.setColor(new Color(0, 0, 0));
					g.fillRect(0, 0, WIDTH, HEIGHT);
					g.setColor(new Color(0, 0, 150));
					g.drawString("Loading:  " + (((int) ((s + 1) * 100 / (Shape.X.ordinal() + 1.0) + (r + 1) / 4.0) * 10.0) / 10.0) + "%", 50, HEIGHT / 8);
					
					for (int p = 0; p < 4; p++) {
						if (images_shapes[s][r][p] == null) {
							images_shapes[s][r][p] = new ImageIcon(loadImage(((s < Shape.X.ordinal() ? "/Texturepacks/Default/" : "/images/") + getShapeName(s)) + "." + image_type)).getImage();
							images_shapes[s][r][p] = createImage(new FilteredImageSource(images_shapes[s][r][p].getSource(), new CropImageFilter(image_resolution * p, image_resolution * r,
									image_resolution, image_resolution)));
						}
					}
				}
			}
			
			for (int i = 0; i < board.length; i++) {
				for (int j = 0; j < board[i].length; j++) {
					board[i][j] = images_shapes[7][0][0];
				}
			}
			
			if (Martomate.backupSaved) {
				boolean playLastGame = Martomate.showConfirmPopup("Do you want to continue the last played game?", "Continue last game?");
				requestFocus();
				
				if (playLastGame) {
					Martomate.getGameBackup(this);
				} else {
					loadNormalValues();
				}
			} else {
				loadNormalValues();
			}
			
			g.setColor(new Color(0, 0, 0));
			g.fillRect(0, 0, getWidth(), getHeight());
			g.setColor(new Color(0, 0, 150));
			g.drawString("Loading: SideBar...", 50, HEIGHT / 8);
			
			sideBar = new SideBar(WIDTH, 0, getWidth() - WIDTH, HEIGHT);
			
			g.setColor(new Color(0, 0, 0));
			g.fillRect(0, 0, getWidth(), getHeight());
			g.setColor(new Color(0, 0, 150));
			g.drawString("Done loading!", 50, HEIGHT / 8);
			
			loadDone = true;
		}
	}
	
	public void loadNormalValues() {
		for (int i = 0; i < board.length; i++) {
			for (int j = 0; j < board[i].length; j++) {
				board[i][j] = images_shapes[7][0][0];
			}
		}
		
		piece = null;
		
		score = 0;
		linesRemoved = 0;
		level = 1;
	}
	
	public static Image loadImage(String path) {
		try {
			return ImageIO.read(Tetris.class.getResource(path));
		} catch (Exception e) {
			Martomate.showPopup("An error occured while loading image:\t\"" + path + "\"\nError-message: " + e.getMessage(), "Error in Tetris line 181");
			System.exit(1);
		}
		return null;
	}
	
	public void paint(Graphics g1) {
		Graphics2D g = (Graphics2D) g1;
		if (!loadDone)
			load(g);
		
		image = new BufferedImage(getWidth(), getHeight(), BufferedImage.TYPE_INT_RGB);
		graphics = image.createGraphics();
		
		paintComponent(graphics);
		g.drawImage(image, 0, 0, null);
		
		if (thread == null)
			repaint();
	}
	
	public void paintComponent(Graphics g1) {
		Graphics2D g = (Graphics2D) g1;
		if (piece == null && thread != null)
			newPieze();
		
		checkEvents(input.key);
		
		if (sideBar != null)
			sideBar.draw(g, this);
		
		if (piece != null) {
			piece.draw(g);
			
			if (ghostPieceEnabled) {
				ghostPiece.draw(g);
			}
		}
		
		Piece.drawFallenPieces(g);
		
		if (thread != null && paused) {
			g.setFont(new Font("Arial Rounded MT Bold", 3, 50));
			g.setColor(new Color(0, 0, 0, 150));
			g.fillRect(0, 0, WIDTH, HEIGHT);
			g.setColor(new Color(0, 150, 150, 200));
			Martomate.drawString(g, "Press P", WIDTH / 5, HEIGHT / 3, true);
		}
		if (thread == null) {
			g.setFont(new Font("Arial Rounded MT Bold", 3, 50));
			g.setColor(new Color(0, 0, 0, 150));
			g.fillRect(0, 0, WIDTH, HEIGHT);
			g.setColor(new Color(0, 150, 150, 200));
			Martomate.drawString(g, "Press", WIDTH / 4, HEIGHT / 3, true);
			Martomate.drawString(g, "SPACE", WIDTH / 5, HEIGHT / 3 + 60, true);
		}
		if (gameOver) {
			g.setFont(new Font("Algerian", 3, 50));
			g.setColor(new Color(50, 0, 0, 100));
			g.drawRect(0, 0, WIDTH, HEIGHT);
			g.setColor(new Color(150, 0, 0));
			Martomate.drawString(g, "Game Over", WIDTH / 5, HEIGHT / 2, true);
		}
		
		g.setColor(new Color(0, 0, 255));
		g.drawRect(0, 0, getWidth() - 1, getHeight() - 1);
	}
	
	public void run() {
		int fallSpeed = 1000;
		timer = new Timer(fallSpeed, new ActionListener() {
			public void actionPerformed(ActionEvent e) {
				movePiezeDown(piece);
			}
		});
		timer.start();
		while (running) {
			try {
				Thread.sleep(1);
			} catch (Exception ex) {
				ex.printStackTrace();
			}
			if (paused || gameOver) {
				timer.stop();
				checkEvents(input.key);
				repaint();
				continue;
			}
			
			timer.start();
			
			repaint();
			
			if(forceScreenShot){
				timer.stop();
				Date date = new Date();
				SimpleDateFormat stf = new SimpleDateFormat("yyyy-MM-dd_HH.mm.ss");
				String str = stf.format(date);
				
				Martomate.showPopup("/" + str, "Date format");
			//	Texture.saveImage(image, "/screenshots/" + Calendar.YEAR + "/" + Calendar.MONTH + "/" + Calendar.DAY_OF_MONTH + "/" + (Calendar.HOUR + ":" + Calendar.MINUTE + ":" + Calendar.SECOND) + ".png");
				
				forceScreenShot = false;
			}
		}
	}
	
	public synchronized void start() {
		if (running)
			return;
		running = true;
		thread = new Thread(this, "Tetris");
		thread.start();
		Martomate.backupSaved = true;
	}
	
	public synchronized void stop() {
		if (!running)
			return;
		running = false;
		timer.stop();
		timer.setRepeats(false);
		System.out.println("Thread stopped");
		Martomate.setGameBackup(this);
		try {
			thread.join();
		} catch (Exception ex) {
			ex.printStackTrace();
			System.exit(1);
		}
	}
	
	boolean pausing = false;
	
	public void checkEvents(boolean key[]) {
		if (new Rectangle(0, 0, getWidth(), getHeight()).contains(new Point(Input.mousePosX, Input.mousePosY)) && Input.isClick) {
			requestFocus();
		}
		
		if (!input.isFocused) {
			paused = true;
			return;
		}
		
		if (Input.isDrag) {
			Point location = getLocation();
			setLocation(location.x + Input.mouseDragX - Input.mouseStartX, location.y + Input.mouseDragY - Input.mouseStartY);
		}
		
		// keyPressed
		if (key[KeyEvent.VK_UP] == true) {
			if (running && !rotating && !gameOver && !paused) {
				piece.rotation++;
				piece.rotateRight();
				if (ghostPieceEnabled) {
					updateGhostPieze();
				}
				rotating = true;
			}
		}
		if (key[KeyEvent.VK_DOWN] == true) {
			if (running && !gameOver && !paused) {
				key[KeyEvent.VK_DOWN] = false;
				movePiezeDown(piece);
			}
		}
		if (key[KeyEvent.VK_RIGHT] && !key[KeyEvent.VK_LEFT]) {
			if (running && !gameOver && !paused) {
				key[KeyEvent.VK_RIGHT] = false;
				moveRight(piece);
				if (ghostPieceEnabled)
					updateGhostPieze();
			}
		}
		if (key[KeyEvent.VK_LEFT] && !key[KeyEvent.VK_RIGHT]) {
			if (running && !gameOver && !paused) {
				key[KeyEvent.VK_LEFT] = false;
				moveLeft(piece);
				if (ghostPieceEnabled)
					updateGhostPieze();
			}
		}
		if (key[KeyEvent.VK_SPACE]) {
			key[KeyEvent.VK_SPACE] = false;
			if (!running) {
				start();
				paused = false;
				if(piece == null)
					newPieze();
			}
		}
		if (key[KeyEvent.VK_ESCAPE]) {
			key[KeyEvent.VK_ESCAPE] = false;
			dispose();
			Martomate.setGameBackup(this);
			new Martomate();
			stop();
		}
		if (key[KeyEvent.VK_D]) {
			key[KeyEvent.VK_D] = false;
			drop(piece);
		}
		if (key[KeyEvent.VK_P]) {
			key[KeyEvent.VK_P] = false;
			if (running && !gameOver && !pausing) {
				paused = !paused;
				pausing = true;
				repaint();
			}
		}
		if(key[KeyEvent.VK_F2]){
			key[KeyEvent.VK_F2] = false;
			forceScreenShot = true;
		}
		if (gameOver) {
			paused = false;
		}
		
		// keyReleased
		if (!key[KeyEvent.VK_UP]) {
			if (rotating) {
				rotating = false;
			}
		}
		
		if (!key[KeyEvent.VK_P]) {
			pausing = false;
		}
		
		if (sideBar != null)
			sideBar.checkEvents(this, key);
		
		if(piece != null)
			piece.update(this);
		
		if(ghostPiece != null)
			ghostPiece.update(this);
	}
	
	public void newPieze() {
		if (piece != null) {
			for (int i = 0; i < 4; i++) {
				int x = piece.x(i);
				int y = piece.y(i);
				
				board[piece.xpos + x][piece.ypos + y] = images_shapes[curShape.ordinal()][piece.rotation % 4][i];
			}
			checkFullLines();
		}
		
		if (nextShape != Shape.X) {
			curShape = nextShape;
			nextShape = getRandomShape(random.nextInt(7));
		} else {
			curShape = getRandomShape(random.nextInt(7));
			nextShape = getRandomShape(random.nextInt(7));
		}
		
		piece = new Piece(0, 0, 0, curShape);
		piece.xpos = board.length / 2 - 1;
		piece.ypos = -piece.minY();
		
		if (ghostPieceEnabled) {
			ghostPiece = piece;
			updateGhostPieze();
		}
		
		if (!movePieze(piece, 0, 0)) {
			gameOver = true;
			Martomate.backupSaved = false;
		}
		
		repaint();
	}
	
	boolean canDropMore = false;
	
	public void movePiezeDown(Piece piezeIn) {
		if (!movePieze(piezeIn, 0, 1)) {
			canDropMore = false;
			if (piezeIn == piece)
				newPieze();
			return;
		}
		
		piezeIn.ypos++;
	}
	
	public void moveLeft(Piece pieze) {
		if (!movePieze(pieze, -1, 0)) {
			return;
		}
		
		pieze.xpos--;
		repaint();
	}
	
	public void moveRight(Piece pieze) {
		if (!movePieze(pieze, 1, 0)) {
			return;
		}
		
		pieze.xpos++;
		repaint();
	}
	
	public void drop(Piece piezeIn) {
		canDropMore = true;
		while (canDropMore) {
			movePiezeDown(piezeIn);
		}
		if (piezeIn == piece)
			checkFullLines();
	}
	
	public boolean movePieze(Piece pieze, int xPlus, int yPlus) {
		boolean canMove = true;
		for (int i = 0; i < 4; i++) {
			int x = pieze.x(i);
			int y = pieze.y(i);
			
			try {
				if (board[pieze.xpos + x + xPlus][pieze.ypos + y + yPlus] == null || board[pieze.xpos + x + xPlus][pieze.ypos + y + yPlus] != images_shapes[7][0][0]) {
					canMove = false;
				}
			} catch (ArrayIndexOutOfBoundsException ex) {
				if (pieze.xpos + x + xPlus + 1 > board.length || pieze.xpos + x + xPlus < 0 || pieze.ypos + y + yPlus + 1 > board[0].length || pieze.ypos + y + yPlus < 0) {
					canMove = false;
				}
			}
		}
		return canMove;
	}
	
	public void updateGhostPieze() {
		if (!ghostPieceEnabled)
			return;
		
		Piece piezeCopy = new GhostPiece(piece.xpos, piece.ypos, piece.rotation, piece.shape);
		piezeCopy.coords = piece.coords;
		drop(piezeCopy);
		ghostPiece = piezeCopy;
	}
	
	int combo = 0;
	int soonLevel = 0;
	
	public void checkFullLines() {
		int numFull = 0;
		for (int j = board[0].length - 1; j >= 0; j--) {
			boolean isFull = true;
			for (int i = 0; i < board.length; i++) {
				if (board[i][j] == images_shapes[7][0][0]) {
					isFull = false;
					break;
				}
			}
			if (isFull) {
				removeFullLine(j++);
				numFull++;
			}
		}
		if (numFull > 0) {
			combo++;
			score += (numFull == 1 ? 10 : numFull == 2 ? 25 : numFull == 3 ? 50 : numFull == 4 ? 85 : 0) * combo;
		} else if (numFull == 0) {
			combo = 0;
		}
		soonLevel += numFull;
		if (soonLevel >= 10) {
			soonLevel -= 10;
			if (level < levelsToWin) {
				level++;
				timer.setDelay((int) (800 - Math.cos((Math.PI / 2) / levelsToWin * level - Math.PI / 2) * 800));
			} else {
				hasWon = true;
			}
		}
	}
	
	public void removeFullLine(int lineNumber) {
		for (int j = lineNumber; j >= 0; j--) {
			for (int i = 0; i < board.length; i++) {
				try {
					if (board[i][j - 1] != null) {
						board[i][j] = board[i][j - 1];
					} else {
						board[i][j] = images_shapes[7][0][0];
					}
				} catch (ArrayIndexOutOfBoundsException e) {
					board[i][j] = images_shapes[7][0][0];
				}
			}
		}
		
		linesRemoved++;
	}
	
	public Shape getRandomShape(int index) {
		if(index >= Shape.values().length - 1){
			Martomate.showPopup("Det blev fel.\tKod:\t" + index, "Error in Tetris line 580");
			System.exit(1);
		}
		
		return Shape.values()[index];
	}
	
	public static String getShapeName(int index) {
		if(index >= Shape.values().length){
			return "";
		}
		
		return Shape.values()[index].letter;
	}
}
