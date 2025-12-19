package martomate;
import java.awt.event.FocusEvent;
import java.awt.event.FocusListener;
import java.awt.event.KeyEvent;
import java.awt.event.KeyListener;
import java.awt.event.MouseEvent;
import java.awt.event.MouseListener;
import java.awt.event.MouseMotionListener;
import java.awt.event.MouseWheelEvent;
import java.awt.event.MouseWheelListener;


public class Input implements KeyListener, MouseListener, MouseMotionListener, MouseWheelListener, FocusListener {
	
	public static boolean isClick = false;
	public static boolean isScroll = false;
	public static boolean isDrag = false;
	public boolean isFocused = false;
	public static int scrollRotation;
	public static int scrollValue;
	public static int mouseButton;
	public boolean[] key = new boolean[68836];
	public static int mousePosX = 0;
	public static int mousePosY = 0;
	public static int mouseStartX = 0;
	public static int mouseStartY = 0;
	public static int mouseDragX = 0;
	public static int mouseDragY = 0;
	
	public Input(){
		for(int i = 0; i < key.length; i++){
			key[i] = false;
		}
	}
	
	public static void reset(){
		isClick = false;
		isScroll = false;
		isDrag = false;
		
		mousePosX = 0;
		mousePosY = 0;
		mouseStartX = 0;
		mouseStartY = 0;
		mouseDragX = 0;
		mouseDragY = 0;
	}
	
	public void mouseWheelMoved(MouseWheelEvent e) {
		scrollRotation = e.getWheelRotation();
		scrollValue = e.getScrollAmount();
		isScroll = true;
	//	System.out.println("MouseWheelRotation: " + scrollRotation);
	}
	//___________________________________________________________________
	
	public void mouseDragged(MouseEvent e) {
		mouseDragX = e.getX();
		mouseDragY = e.getY();
		mouseButton = e.getButton();
		isDrag = true;
		isClick = true;
	//	System.out.println((mouseStartX - mouseDragX) + "  " + (mouseStartY - mouseDragY) + "   Drag");
	}

	public void mouseMoved(MouseEvent e) {
	//	System.out.println("mouseX: " + e.getX() + "  mouseY: " + e.getY());
		mousePosX = e.getX();
		mousePosY = e.getY();
		isScroll = false;
	}

	public void mousePressed(MouseEvent e) {
		mouseStartX = e.getX();
		mouseStartY = e.getY();
		mouseButton = e.getButton();
		isClick = true;
	//	System.out.println(mouseStartX + "  " + mouseStartY + "  Press");
	}

	public void mouseReleased(MouseEvent e) {
		mouseDragX = 0;
		mouseDragY = 0;
		mouseStartX = 0;
		mouseStartY = 0;
		isClick = false;
		isDrag = false;
	}
	//___________________________________________________________________

	public void keyPressed(KeyEvent e) {
		key[e.getKeyCode()] = true;
		e.consume();
	//	System.out.println("key: " + e.getKeyChar());
	}
	
	public void keyReleased(KeyEvent e) {
		key[e.getKeyCode()] = false;
	}

	public void focusGained(FocusEvent e) {
		this.isFocused = true;
		
	}

	public void focusLost(FocusEvent e) {
		for(int i = 0; i < key.length; i++){
			key[i] = false;
		}
		this.isFocused = false;
	}
	//___________________________________________________________________
	
	public void mouseClicked(MouseEvent e) {}

	public void mouseEntered(MouseEvent e) {}

	public void mouseExited(MouseEvent e) {}
	
	public void keyTyped(KeyEvent e) {}
}
