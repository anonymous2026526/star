unforgeability:
	proverif -lib ./protocol/core.pvl -lib ./protocol/server.pvl -lib ./protocol/honest_server.pvl -lib ./protocol/user.pvl  ./unforgeability.pv 

rate_limiting:
	proverif -lib ./protocol/core.pvl -lib ./protocol/server.pvl -lib ./protocol/honest_server.pvl -lib ./protocol/user.pvl  ./rate-limitability.pv 

unlinkability: 
	proverif -lib ./protocol/core.pvl -lib ./protocol/server.pvl -lib ./protocol/malicious_server.pvl -lib ./protocol/user.pvl ./unlinkability.pv 

unlinkability_reachability:
	proverif -lib ./protocol/core.pvl -lib ./protocol/server.pvl -lib ./protocol/malicious_server.pvl -lib ./protocol/user.pvl ./unlinkability_reachability.pv 

fail_table:
	proverif -lib ./protocol/core.pvl -lib ./protocol/server.pvl -lib ./protocol/honest_server.pvl -lib ./protocol/user.pvl  ./fail-table.pv
