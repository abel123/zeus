export const StockOptionTable = () => {
    return (
        <div className="grid grid-cols-12 grid-rows-9 text-center border-0 border-slate-100 bg-white h-full">
            {/* header */}
            <div className="col-span-2 row-span-1 border-2"></div>
            <div className="col-span-2 row-span-1 border-2">Price</div>
            <div className="col-span-2 row-span-1 border-2">Put - 1</div>
            <div className="col-span-2 row-span-1 border-2">Put + 1</div>
            <div className="col-span-2 row-span-1 border-2">Call - 1</div>
            <div className="col-span-2 row-span-1 border-2">Call + 1</div>
            <div className="border-slate-400 col-span-12 grid grid-cols-12">
                {/* k 30 */}
                <div className="col-span-2 row-span-1 border-2">30分钟-120</div>
                <div className="col-span-2 row-span-1 border-2">Price</div>
                <div className="col-span-2 row-span-1 border-2">Put - 1</div>
                <div className="col-span-2 row-span-1 border-2">Put + 1</div>
                <div className="col-span-2 row-span-1 border-2">Call - 1</div>
                <div className="col-span-2 row-span-1 border-2">Call + 1</div>
                {/* k 60 */}
                <div className="col-span-2 row-span-1 border-2">60分钟-120</div>
                <div className="col-span-2 row-span-1 border-2">Price</div>
                <div className="col-span-2 row-span-1 border-2">Put - 1</div>
                <div className="col-span-2 row-span-1 border-2">Put + 1</div>
                <div className="col-span-2 row-span-1 border-2">Call - 1</div>
                <div className="col-span-2 row-span-1 border-2">Call + 1</div>
            </div>
            <div className="border-blue-400 col-span-12 grid grid-cols-12">
                {/* k 30 */}
                <div className="col-span-2 row-span-1 border-blue-200 border-2">30分钟-200</div>
                <div className="col-span-2 row-span-1 border-blue-200 border-2">Price</div>
                <div className="col-span-2 row-span-1 border-blue-200 border-2">Put - 1</div>
                <div className="col-span-2 row-span-1 border-blue-200 border-2">Put + 1</div>
                <div className="col-span-2 row-span-1 border-blue-200 border-2">Call - 1</div>
                <div className="col-span-2 row-span-1 border-blue-200 border-2">Call + 1</div>
                {/* k 60 */}
                <div className="col-span-2 row-span-1 border-blue-200 border-2">60分钟-200</div>
                <div className="col-span-2 row-span-1 border-blue-200 border-2">Price</div>
                <div className="col-span-2 row-span-1 border-blue-200 border-2">Put - 1</div>
                <div className="col-span-2 row-span-1 border-blue-200 border-2">Put + 1</div>
                <div className="col-span-2 row-span-1 border-blue-200 border-2">Call - 1</div>
                <div className="col-span-2 row-span-1 border-blue-200 border-2">Call + 1</div>
            </div>
        </div>
    );
};
