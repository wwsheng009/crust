
struct POWER
{
	int nFlags; //除权的种类，0为“先送后配”，1为“先配后送”，2为“股本不变”
	int nTime;
	float fGive;		//每股送
	float fAllocate;	//每股配
	float fAllocatePrice;//配股价
	float fDividend;	//每股红利
};