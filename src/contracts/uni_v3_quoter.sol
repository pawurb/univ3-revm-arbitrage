//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IUniV3Pool {
    function swap(
        address recipient,
        bool zeroForOne,
        int256 amountSpecified,
        uint160 sqrtPriceLimitX96,
        bytes calldata data
    ) external returns (int256 amount0, int256 amount1);
}

contract UniV3Quoter {
    function uniswapV3SwapCallback(
        int256 amount0Delta,
        int256 amount1Delta,
        bytes calldata _data
    ) external {
        revert(string(abi.encode(amount0Delta, amount1Delta)));
    }

    function getAmountOut(
        address pool,
        bool zeroForOne,
        uint256 amountIn
    ) external {
        uint160 sqrtPriceLimitX96 = (
            zeroForOne
                ? 4295128749
                : 1461446703485210103287273052203988822378723970341
        );

        IUniV3Pool(pool).swap(
            address(1),
            zeroForOne,
            int256(amountIn),
            sqrtPriceLimitX96,
            ""
        );
    }
}
