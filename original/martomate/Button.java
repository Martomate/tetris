package martomate;

import java.awt.Color;
import java.awt.Font;
import java.awt.Graphics2D;
import java.awt.Point;
import java.awt.Rectangle;

public class Button extends Rectangle {
	private static final long serialVersionUID = 1L;
	
	private double brightness = 0.0;
	private Color color;
	private String text;
	private boolean doAction = false;
	private boolean isClicked = false;
	
	public Button(int x, int y, int width, int height, String text) {
		setBounds(x, y, width, height);
		this.text = text;
		
		color = new Color(0, 255, 0);
	}
	
	public Button(int x, int y, int width, int height, String text, Color color) {
		setBounds(x, y, width, height);
		this.text = text;
		
		this.color = color;
	}
	
	public void draw(Graphics2D g) {
		double colorFallRed = color.getRed();
		double colorFallGreen = color.getGreen();
		double colorFallBlue = color.getBlue();
		int cR, cG, cB;
		int range = height;
		
		for (int i = 0; i < range; i++) {
			cR = 0 + (int) (i * ((brightness) * colorFallRed) / range);
			cG = 0 + (int) (i * ((brightness) * colorFallGreen) / range);
			cB = 0 + (int) (i * ((brightness) * colorFallBlue) / range);
			
			if (cR > 255)
				colorFallRed *= 0.5;
			if(cG > 255)
				colorFallGreen *= 0.5;
			if(cB > 255)
				colorFallBlue *= 0.5;
			
			g.setColor(new Color(Math.max(Math.min(cR, 255), 0), Math.max(Math.min(cG, 255), 0), Math.max(Math.min(cB, 255), 0)));
			g.fillOval((int) (x + i + width / 2 - brightness * (width / 2)), (int) (y + i + height / 2 - brightness * (height / 2)), (int) (width - i * 2 - width + brightness * width), (int) (height - i * 2 - height + brightness * height));
		}
		
		
		g.setColor(this.color);
		g.setFont(new Font("Arial Rounded MT Bold", 2, width / 5));
		
		Martomate.drawString(g, text, x + (int) (width / 2.0 - (Martomate.getTextBounds(g, text).getWidth() / 2)), y + (int) (height / 2.0 + (Martomate.getTextBounds(g, text).getHeight() / 3)), true);
	}
	
	public void update() {
		brightness = hover();
	}
	
	public double hover() {
		double reLightSpeed = 20 / 1000.0;
		
		if (contains(new Point(Input.mousePosX, Input.mousePosY)) && !Input.isDrag) {
			if (!Input.isClick && !isClicked) {
				if (brightness <= 1.2) {
					brightness += reLightSpeed * (brightness + reLightSpeed * (20 / (brightness + 0.1))) * 2;
				}
			} else {
				isClicked = true;
			}
			
			if (isClicked) {
				if (brightness >= -1.2) {
					brightness -= reLightSpeed * (brightness + reLightSpeed * (20 / (brightness + 0.1))) * 2;
				} else {
					doAction = true;
				}
			}
			brightness = (brightness != 0.0 ? ((int) (brightness * 1000)) / 1000.0 : 0.0);
		} else {
			brightness = (brightness != 0.0 ? ((int) (brightness * 100)) / 100.0 : 0.0);
			if (brightness > 0.0) {
				brightness -= reLightSpeed;
			}
			if (brightness < 0.0) {
				brightness += reLightSpeed;
			}
			if (brightness < reLightSpeed && brightness > -reLightSpeed) {
				brightness = 0.0;
			}
			if(brightness > 1.2)
				brightness = 1.2;
			if(brightness < -1.2)
				brightness = -1.2;
		}
		return brightness;
	}
	
	public boolean getDoAction() {
		return doAction;
	}
	
}
