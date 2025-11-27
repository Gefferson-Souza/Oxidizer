import { add } from './utils';
import * as math from './math';
import defaultExport from './default';

function main() {
    add(1, 2);
    math.sub(3, 4);
    defaultExport();
}
