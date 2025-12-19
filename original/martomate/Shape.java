package martomate;
public enum Shape {

	O("O"), I("I"), J("J"), L("L"), Z("Z"), S("S"), T("T"), X("X"); // X = EMPTY
	
	public String letter;
	
	Shape(String letter){
		this.letter = letter;
	}
}
