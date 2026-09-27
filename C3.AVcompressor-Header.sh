# ==============================================================================
# C3.AVcompressor
# by C3net Development
#
# ANSI Color Codes
if [ -t 1 ]; then
    BOLD='\033[1m'   
    GREEN='\033[0;32m'
    RED='\033[0;31m'
    YELLOW='\033[1;33m'
    CYAN='\033[0;36m'
    BLUE='\033[0;34m'
    GRAD_ORANGE='\033[38;2;255;135;0m'
    GRAD_RED='\033[38;2;230;35;35m'
    GRAD_DARKRED='\033[38;2;140;20;20m'
    NC='\033[0m' # No Color
else
    BOLD=''
    GREEN=''
    RED='' 
    YELLOW=''
    CYAN=''
    BLUE=''
    GRAD_ORANGE=''
    GRAD_RED=''
    GRAD_DARKRED=''
    NC=''
fi
    
# Print Header Banner
show_banner() {
    if [ -t 1 ]; then
        clear 2>/dev/null || true
    fi
    echo -e "${GRAD_ORANGE}░█▀▀░▀▀█░█▀█░█▀▀░▀█▀${NC}"
    echo -e "${GRAD_RED}░█░░░░▀▄░█░█░█▀▀░░█░${NC}"
    echo -e "${GRAD_DARKRED}░▀▀▀░▀▀░░▀░▀░▀▀▀░░▀░${NC}"
    echo ""
    echo -e "${BOLD}C3.AVcompressor${NC}"
    echo "--------------------------------------------------------"
    echo ""
}
