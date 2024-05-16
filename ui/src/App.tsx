import { useEffect, useState } from "react";
import {
  Alert,
  Button,
  CircularProgress,
  CssBaseline,
  Snackbar,
  Stack,
  TextField,
  Typography,
  useTheme,
} from "@mui/material";

function App() {
  const theme = useTheme();
  const [password, setPassword] = useState<string | null>(null);
  const [passwordHint, setPasswordHint] = useState<string | null>(
    "Pound of ____, extra ____",
  );
  const [restarting, setRestarting] = useState<boolean>(false);

  const [alertOpen, setAlertOpen] = useState<boolean>(false);
  const [alertMessage, setAlertMessage] = useState<string | null>(null);
  const [alertIsError, setAlertIsError] = useState<boolean>(false);

  useEffect(() => {
    const run = async () => {
      const response = await fetch("/minecraft/api/password_hint");
      if (response.status === 200) {
        const text = await response.text();
        setPasswordHint(JSON.parse(text));
      }
    };

    run();
  }, []);

  const sendRestartRequest = async () => {
    setRestarting(true);

    const response = await fetch("/minecraft/api/restart", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ password }),
    });

    setRestarting(false);

    if (response.status === 200) {
      setAlertIsError(false);
      setAlertMessage("Server successfully restarted");
      setAlertOpen(true);
    } else {
      setAlertIsError(true);
      setAlertMessage("Error restarting server");
      setAlertOpen(true);
    }
  };

  const onAlertClose = () => {
    setAlertOpen(false);
  };

  return (
    <>
      <CssBaseline />

      <Stack direction="column" spacing={2} padding={theme.spacing(2)}>
        <Typography variant="h4">Minecraft Control Panel</Typography>

        <Stack direction="column" spacing={1}>
          <Stack direction="row" spacing={1} alignItems="center">
            <TextField
              type="password"
              label="Password"
              value={password}
              onChange={(event: React.ChangeEvent<HTMLInputElement>) => {
                setPassword(event.target.value);
              }}
            />

            <Button
              variant="contained"
              disabled={
                restarting || password === null || password.trim() === ""
              }
              onClick={() => sendRestartRequest()}
            >
              Restart
            </Button>

            {restarting && <CircularProgress variant="indeterminate" />}
          </Stack>
          <Typography variant="subtitle2" color={theme.palette.text.disabled}>
            Hint: {passwordHint}
          </Typography>
        </Stack>
      </Stack>

      <Snackbar open={alertOpen} autoHideDuration={5000} onClose={onAlertClose}>
        <Alert
          onClose={onAlertClose}
          severity={alertIsError ? "error" : "success"}
          variant="filled"
          sx={{ width: "100%" }}
        >
          {alertMessage}
        </Alert>
      </Snackbar>
    </>
  );
}

export default App;
