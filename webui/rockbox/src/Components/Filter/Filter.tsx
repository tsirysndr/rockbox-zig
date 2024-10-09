/* eslint-disable @typescript-eslint/no-explicit-any */
import { Theme, useTheme } from "@emotion/react";
import { Input } from "baseui/input";
import { FC, useEffect } from "react";
import { Controller, useForm } from "react-hook-form";
import Search from "../Icons/Search";

export type FilterProps = {
  placeholder?: string;
  onChange: (value: string) => void;
};

const Filter: FC<FilterProps> = ({ placeholder = "Filter", onChange }) => {
  const theme = useTheme();
  const { control, watch } = useForm({
    defaultValues: {
      filter: "",
    },
  });

  useEffect(() => {
    const subscription = watch((value, { name, type }) => {
      if (type === "change") {
        onChange(value[name!] as string);
      }
    });
    return () => subscription.unsubscribe();
  }, [onChange, watch]);

  return (
    <>
      <Controller
        render={({ field }) => (
          <Input
            {...(field as any)}
            startEnhancer={<Search />}
            placeholder={placeholder}
            overrides={styles.filter(theme)}
          />
        )}
        control={control}
        name="filter"
        rules={{ required: true }}
      />
    </>
  );
};

const styles = {
  filter: (theme: Theme) => ({
    Root: {
      style: {
        height: "40px",
        width: "222px",
        borderTopWidth: "1.5px !important",
        borderBottomWidth: "1.5px !important",
        borderLeftWidth: "1.5px !important",
        borderRightWidth: "1.5px !important",
        borderTopColor: "rgba(82, 82, 82, 0.0625) !important",
        borderBottomColor: "rgba(82, 82, 82, 0.0625) !important",
        borderLeftColor: "rgba(82, 82, 82, 0.0625) !important",
        borderRightColor: "rgba(82, 82, 82, 0.0625) !important",
        borderTopLeftRadius: "20px !important",
        borderTopRightRadius: "20px !important",
        borderBottomLeftRadius: "20px !important",
        borderBottomRightRadius: "20px !important",
        backgroundColor: theme.colors.searchBackgroundAlt,
      },
    },
    Input: {
      style: {
        backgroundColor: theme.colors.searchBackgroundAlt,
        fontSize: "14px",
        color: theme.colors.text,
      },
    },
    InputContainer: {
      style: {
        backgroundColor: theme.colors.searchBackgroundAlt,
      },
    },
    StartEnhancer: {
      style: {
        backgroundColor: theme.colors.searchBackgroundAlt,
        paddingTop: "0px",
        paddingBottom: "0px",
        paddingLeft: "0px",
        paddingRight: "0px",
      },
    },
  }),
};

export default Filter;
