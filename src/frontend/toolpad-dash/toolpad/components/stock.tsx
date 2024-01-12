import * as React from "react";
import { Typography } from "@mui/material";
import { createComponent } from "@mui/toolpad/browser";

export interface stockProps {
  msg: string;
}

function stock({ msg }: stockProps) {
  return <Typography>{msg}</Typography>;
}

export default createComponent(stock, {
  argTypes: {
    msg: {
      type: "string",
      default: "Hello world!",
    },
  },
});
