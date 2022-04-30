// #define AFX_LJISHU_H__A3E1ADE5_B7F2_11D1_83EC_0000E8593F1A__INCLUDED_

// #if _MSC_VER > 1000
// #pragma once
// #endif // _MSC_VER > 1000
// // CFormularContent.h : header file
// //
// #define Formu_Array1 CArray<CFormularContent*,CFormularContent*>
// #define LEN_BYTE	240


// struct FLOAT11
// {
// 	float fValue[11];
// };
/*enum ENUM_ENTER_POINT
{
	MIDDLE = 0;//本周期中价
	MIDDLE ;//本周期中价
	MIDDLE ;//本周期中价
	MIDDLE ;//本周期中价
	MIDDLE ;//本周期中价

}*/


/////////////////////////////////////////////////////////////////////////////
// CFormularContent command target
class CSharesDocument;


 class AFX_EXT_CLASS CFormularContent : public CObject
{
	// protected constructor used by dynamic creation

// Attributes
public:
	CFormularContent();
	// union
	// {
		struct a
		{
			//最大值
			float		max[8];

			//最小值
			float		min[8];

			//缺省值
			//0:HS 1:MIN1 2:MIN5 3:MIN15 4:MIN30 5:MIN60 6:DAY (MANYDAY ) 7:WEEK
			//8:MONTH 9:YEAR 
			float		defaultVal[8];
			//				float		defaultValArray[11][8];	//缺省值


			float	stepLen[8];		//步长
			float	posX[8];		//Y坐标轴的横线的数值
			float	fReserved;		//

			//周期类型
			//0:HS 1:MIN1 2:MIN5 3:MIN15 4:MIN30 5:MIN60 6:DAY  7:WEEK
			//8:MONTH 9:YEAR 10:MANYDAY 
			int nKindPeriod;		
			
			BYTE	nPeriodsUsed[16];	//有效周期

			BOOL	isSystem;			//是否是系统指标
			BOOL	isOfen;				//是否是常用指标
			BOOL	isProtected;		//是否密码保护

			BYTE	isMainFiguer;		//是否是主图叠加（0为副图，1为主图，2为不可用）
			BYTE	numPara;			//参数个数
			BYTE 	posFlag;			//Y坐标轴的横线的式样（0为自动，1为定制（可有7条），2为不可用）
			BYTE 	isAdditionalCond;	//是否含有附加条件

			BYTE	bYHParam[8];		//是否优化参数
			BYTE	bYH;				//是否自动优化参数
			BYTE	bRightBox;			//是否是右边的指标

			//是否是来自服务器的指标，如果m_nIsFromServer = 33134999是来自服务器的指标
			int		m_nIsFromServer;//

		};
	// 	BYTE btMemData[LEN_BYTE];
	// };
	ADDITIONAL_BUYSELL * pAdditionalBS;				//买卖附加条件
	CArray<FLOAT11, FLOAT11&>	defaultValArray;	//不同时间周期的缺省参数

	CString	name;				//技术分析指标名字(最多9个字符)
	CString	password;			//密码
	CString	explainBrief;		//指标说明
	CString	explainParam;		//参数说明

	CString	namePara[8];		//参数名字（最多15个字符)
	CString	fomular;			//公式
	CString	help;				//帮助注释

	//-----专家系统中的买进和卖出条件---------
	CString buyStr;				//买入的条件
	CString sellStr;			//卖出的条件
	CString buyStrKong;			//买入的条件
	CString sellStrKong;		//卖出的条件
	// add ,3.13
	CString	subKindIndex;		//技术分析指标种类,缺省值是 ""
	CString	subKindIndexTime;	//技术分析指标种类
	CString	strReserved;		//

	static BOOL m_bVer20;
	static CStringArray m_strArrayKind[4];
	float  defaultValYH[8];		//优化缺省值


// Operations
public:
	
	static float GetParamDataEach(int iIndex, int nKlineType, CFormularContent* pJishu);
	static void DeleteKindName(CString s, int nKind);
	static void AddKindName(CString s, int nKind = 0);
	static void ReadWriteIndex(CSharesDocument *pDoc, int nKind = 0, BOOL bRead = TRUE);
	static BOOL ExportFormular(Formu_Array1* pArr, int nArr, CString fileName, BOOL bCompleteSecret, BOOL bPassword, CString strPassword);
	static BOOL InstallIndicator(CString sFilePathName, BOOL bPromp = FALSE, BOOL bFromServer = TRUE);
	static BOOL InstallIndicatorCwd(CString sFilePathName, BOOL bPromp = FALSE, BOOL bFromServer = TRUE);
	
	CString GetFormular();
	BOOL IsValid();
	void SecretForm(CString& s, BOOL bLoad);
	void AddDefaultValToArray();
	
	void InitDefaultValArray();
	
	void SerializeData(CArchive& ar);
	void SetData(CFormularContent * data);



	virtual ~CFormularContent();
	// Overrides
		// ClassWizard generated virtual function overrides
		//{{AFX_VIRTUAL(CFormularContent)
		//}}AFX_VIRTUAL

	// Implementation
protected:

	virtual void Serialize(CArchive& ar);
	DECLARE_SERIAL(CFormularContent)

	// Generated message map functions
	//{{AFX_MSG(CFormularContent)
		// NOTE - the ClassWizard will add and remove member functions here.
	//}}AFX_MSG


};

/////////////////////////////////////////////////////////////////////////////

//{{AFX_INSERT_LOCATION}}
// Microsoft Visual C++ will insert additional declarations immediately before the previous line.

// #endif // !defined(AFX_LJISHU_H__A3E1ADE5_B7F2_11D1_83EC_0000E8593F1A__INCLUDED_)
