import { useState, Dispatch, SetStateAction } from "react";

type FloatInputProps = {
  value: string;
  setValue: Dispatch<SetStateAction<string>>;
};

export default function FloatInput({ value, setValue }: FloatInputProps) {
  return (
    <input
      type="text"
      className="text-3xl inline text-right mr-5 bg-zinc-100 dark:bg-zinc-900"
      value={value === "0.0" ? "" : value}
      placeholder="0.0"
      style={{ maxWidth: "250px" }}
      onChange={(e) => {
        const re = /^[0-9\b]+[.]?[0-9\b]*$/;
        if (e.target.value === "" || re.test(e.target.value)) {
          setValue(e.target.value);
        }
      }}
    />
  );
}
