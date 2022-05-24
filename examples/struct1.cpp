///////////////////////////////////////////////////////////////////
// 函 数 名：OnDayKline()
// 功能说明：转换K线类型的菜单命令处理
// 入口参数：无
// 返回参数：无返回数
/////////////////////////////////////////////////////////////////////////////
void CTaiShanKlineShowView::OnDayKline()
{
	m_nKlineKind2 = 5;

	bTrackCurcorBgn = FALSE;

	CRect r;
	GetClientRect(r);
	OnSizeMy(r.right - r.left, r.bottom - r.top);

	ShowAll(m_sharesSymbol);
}