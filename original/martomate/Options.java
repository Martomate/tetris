package martomate;

import java.awt.Choice;
import java.awt.Color;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.Point;
import java.awt.Rectangle;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.awt.event.KeyEvent;
import java.awt.image.BufferedImage;
import java.beans.PropertyChangeEvent;
import java.beans.PropertyChangeListener;

import javax.swing.JButton;
import javax.swing.JFrame;
import javax.swing.JPanel;
import javax.swing.JTextField;


public class Options extends JFrame implements ActionListener, PropertyChangeListener{
	private static final long serialVersionUID = 1L;

	public Input input;
	
	private Button quitButton;
	
	private BufferedImage image;
	private Graphics2D graphics;
	
	public JButton button1;
	public JTextField textField1;
	public JPanel panel = new JPanel();
	public Choice resolution;
	
	public Options(){
		setTitle("Options");
		setSize(400, 300);
		setResizable(false);
		setUndecorated(true);
		setLocationRelativeTo(null);
		setBackground(new Color(0,0,0));
		setLayout(null);
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
	}
	
	public void load(){
		quitButton = new Button(20, getHeight() - 50, 80, 30, "Close");
		
		button1 = new JButton("Texture");
		button1.setBounds(50, 20, 80, 20);
		button1.setBackground(Color.BLACK);
		button1.setForeground(Color.GREEN);
		button1.addActionListener(this);
		
		textField1 = new JTextField("");
		//("width * height");
		textField1.setBounds(50, 50, 80, 20);
		textField1.setBackground(Color.BLACK);
		textField1.setForeground(Color.GREEN);
		textField1.addActionListener(this);
		
		resolution = new Choice();
		resolution.setBounds(50, 80, 120, 15);
		resolution.addItem("160 * 320");
		resolution.addItem("320 * 640");
		resolution.addItem("640 * 1280");
		resolution.select(1);
		resolution.setBackground(Color.BLACK);
		resolution.setForeground(Color.GREEN);
		resolution.addPropertyChangeListener(this);
		
		add(button1);
		add(textField1);
		add(resolution);
	}
	
	public void paint(Graphics g1){
		Graphics2D g = (Graphics2D) g1;
		
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
		g.setColor(new Color(0,0,0));
		g.fillRect(0, 0, getWidth(), getHeight());
		
		textField1.update(g);
		
		g.setColor(new Color(0,0,255));
		g.drawRect(0, 0, getWidth() - 1, getHeight() - 1);
		
		
		if(quitButton != null)
			quitButton.draw(g);
		
		checkEvents(input.key);
	}
	
	public void checkEvents(boolean key[]){
		if(new Rectangle(0, 0, getWidth(), getHeight()).contains(new Point(Input.mousePosX, Input.mousePosY)) && Input.isClick){
			requestFocus();
		}
		
		if(!input.isFocused)
			return;
		
		
		if(Input.isDrag){
			Point location = getLocation();
			setLocation(location.x + Input.mouseDragX - Input.mouseStartX, location.y + Input.mouseDragY - Input.mouseStartY);
		}
		
		if(contains(new Point(Input.mousePosX, Input.mousePosY)) && Input.isClick){
			requestFocus();
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
	
	public void actionPerformed(ActionEvent e){
		if(e.getSource() == button1){
			dispose();
			new Texture();
		}else if(e.getSource() == textField1 && input.key[KeyEvent.VK_ENTER]){
			requestFocus();
		}
	}
	
	public void propertyChange(PropertyChangeEvent e){
		if(e.getSource() == resolution){
			String str = resolution.getSelectedItem();
			int pos = str.indexOf(" * ");
			String wStr = str.substring(0, pos);
			String hStr = str.substring(pos + 3, str.length());
			int wInt = Integer.parseInt(wStr);
			int hInt = Integer.parseInt(hStr);
			
			Tetris.WIDTH = wInt;
			Tetris.HEIGHT = hInt;
		}
	}
}
